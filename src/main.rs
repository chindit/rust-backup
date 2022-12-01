use flate2::write::{GzEncoder};
use flate2::Compression;
use std::fs::File;
use std::error;
use log::{info, error, LevelFilter};
use chrono::{DateTime, Utc};
use ftp::FtpStream;
use config::Config;
use std::fs;

fn main() -> Result<(), Box<dyn error::Error>> {

	// Setup logger
	systemd_journal_logger::init().unwrap();
	log::set_max_level(LevelFilter::Info);

	// Import config
	let settings = Config::builder().add_source(config::File::with_name("/etc/plex-backup.json")).build().unwrap();

	let source_directory = settings.get::<String>("sourceDirectory").unwrap();
	let mut temp_directory = settings.get::<String>("tempDirectory").unwrap();

	let ftp_server = settings.get::<String>("ftp.server").unwrap();
	let ftp_username = settings.get::<String>("ftp.username").unwrap();
	let ftp_password = settings.get::<String>("ftp.password").unwrap();

	if !temp_directory.ends_with('/') {
		temp_directory.push('/');
	}

	info!("Backup started");

	let archive: String = compress(&source_directory, &temp_directory);

	upload(&archive, &ftp_server, &ftp_username, &ftp_password);

	delete_file(&archive);

	Ok(())
}

fn compress(source_directory: &str, temp_directory: &str) -> String {
	println!("Starting compression");

	let date: DateTime<Utc> = Utc::now();
	let filename = format!("{}plex_{}.tar.gz", temp_directory, date.format("%d%m%Y"));

	let tar_gz = match File::create(&filename) {
		Ok(file) => file,
		Err(error) => {
			error!("Unable to create file {}.  Returned error is {}", &filename, error);
			panic!("Unable to create archive");
		}
	};
	let enc = GzEncoder::new(tar_gz, Compression::default());
	let mut tar = tar::Builder::new(enc);
	match tar.append_dir_all("", source_directory) {
		Ok(()) => (),
		Err(error) => {
			error!("Unable to append dir {}.  Returned error is {}", &source_directory, error);
			tar.finish().expect("Unable to close temp file");
			delete_file(&filename);
			panic!("Unable to locate dir.");
		}
	}
	tar.finish().unwrap();

	println!("Compression success");

	filename
}

fn upload(filename: &str, server: &str, username: &str, password: &str) {
	println!("Starting upload");

	let mut file: File = File::open(filename).expect("Could not open created archive");
	let mut ftp_stream = FtpStream::connect(server).expect("Could not connect to ftp");
	ftp_stream.login(username, password).expect("Could not login on ftp");
	ftp_stream.put(filename, &mut file).expect("Could not upload file to ftp");

	println!("Successfully uploaded file");

	// Terminate the connection to the server.
	let _ = ftp_stream.quit();
}

fn delete_file(filename: &str) {
	println!("Deleting temporary file {}", filename);
	fs::remove_file(filename).expect("Temp file couldn't be removed");
}