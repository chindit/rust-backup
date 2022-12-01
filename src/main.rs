mod config;

use crate::config::{Config, ConfigDirectory, ConfigFtp};
use chrono::{DateTime, Utc};
use flate2::write::GzEncoder;
use flate2::Compression;
use ftp::FtpStream;
use log::{error, info, LevelFilter};
use std::error;
use std::fs;
use std::fs::File;

fn main() -> Result<(), Box<dyn error::Error>> {
    // Setup logger
    systemd_journal_logger::init().unwrap();
    log::set_max_level(LevelFilter::Info);

    // Import config
    let config = Config::from_file("/etc/plex-backup.json")?;

    info!("Backup started");

    let archive: String = compress(&config.directory);

    upload(&archive, &config.ftp);

    delete_file(&archive);

    Ok(())
}

fn compress(config: &ConfigDirectory) -> String {
    println!("Starting compression");

    let date: DateTime<Utc> = Utc::now();
    let filename = format!("{}plex_{}.tar.gz", &config.temp, date.format("%d%m%Y"));

    let tar_gz = match File::create(&filename) {
        Ok(file) => file,
        Err(error) => {
            error!("Unable to create file {}.  Returned error is {}", &filename, error);
            panic!("Unable to create archive");
        }
    };
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    match tar.append_dir_all("", &config.source) {
        Ok(()) => (),
        Err(error) => {
            error!("Unable to append dir {}.  Returned error is {}", &config.source, error);
            tar.finish().expect("Unable to close temp file");
            delete_file(&filename);
            panic!("Unable to locate dir.");
        }
    }
    tar.finish().unwrap();

    println!("Compression success");

    filename
}

fn upload(filename: &str, config: &ConfigFtp) {
    println!("Starting upload");

    let mut file: File = File::open(filename).expect("Could not open created archive");
    let mut ftp_stream = FtpStream::connect(&config.server).expect("Could not connect to ftp");
    ftp_stream.login(&config.username, &config.password).expect("Could not login on ftp");
    ftp_stream.put(filename, &mut file).expect("Could not upload file to ftp");

    println!("Successfully uploaded file");

    // Terminate the connection to the server.
    let _ = ftp_stream.quit();
}

fn delete_file(filename: &str) {
    println!("Deleting temporary file {}", filename);
    fs::remove_file(filename).expect("Temp file couldn't be removed");
}
