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
    let settings = Config::builder()
        .add_source(config::File::with_name("/etc/plex-backup.json"))
        .build()
        .unwrap();

    info!("Backup started");

    let archive: String = compress(settings
        .get::<String>("sourceDirectory")
        .unwrap());

    upload(
        archive.to_string(),
        settings.get::<String>("ftp.server").unwrap(),
        settings.get::<String>("ftp.username").unwrap(),
        settings.get::<String>("ftp.password").unwrap()
    );

    delete_file(archive.to_string());

    Ok(())
}

fn compress(source_directory: String) -> String {
    println!("Starting compression");

    let date: DateTime<Utc> = Utc::now();
    let mut filename = String::new();
    filename.push_str("plex_");
    filename.push_str(&format!("{}", date.format("%d%m%Y")));
    filename.push_str(".tar.gz");

    let tar_gz = File::create(&filename);
    let tar_gz = match tar_gz {
        Ok(file) => file,
        Err(error) => {
            error!("Unable to create file {}.  Returned error is {}", &filename, error);
            panic!("Unable to create archive");
        }
    };
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    match tar.append_dir_all("", &source_directory) {
        Ok(()) => (),
        Err(error) => {
            error!("Unable to append dir {}.  Returned error is {}", &source_directory, error);
            tar.finish().expect("Unable to close temp file");
            delete_file(filename);
            panic!("Unable to locate dir.");
        }
    }
    tar.finish().unwrap();

    println!("Compression success");

    return filename;
}

fn upload(filename: String, server: String, username: String, password: String) {
    println!("Starting upload");

    let mut file: File = File::open(&filename).unwrap();
    let mut ftp_stream = FtpStream::connect(&server).unwrap();
    let _ = ftp_stream.login(&username, &password).unwrap();
    let _ = ftp_stream.put(&filename, &mut file);
    println!("Successfully uploaded file");

    // Terminate the connection to the server.
    let _ = ftp_stream.quit();
}

fn delete_file(filename: String)
{
    println!("Deleting temporary file {}", filename);
    fs::remove_file(filename).expect("Temp file couldn't be removed");
}