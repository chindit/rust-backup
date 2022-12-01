use std::error::Error;

pub(crate) struct Config {
    pub(crate) directory: ConfigDirectory,
    pub(crate) ftp: ConfigFtp,
}

pub(crate) struct ConfigDirectory {
    pub(crate) source: String,
    pub(crate) temp: String,
}

pub(crate) struct ConfigFtp {
    pub(crate) server: String,
    pub(crate) username: String,
    pub(crate) password: String,
}

impl Config {
    pub(crate) fn from_file(file_name: &str) -> Result<Config, Box<dyn Error>> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(file_name))
            .build()?;

        let source_directory = settings.get::<String>("sourceDirectory")?;
        let mut temp_directory = settings.get::<String>("tempDirectory")?;

        let ftp_server = settings.get::<String>("ftp.server")?;
        let ftp_username = settings.get::<String>("ftp.username")?;
        let ftp_password = settings.get::<String>("ftp.password")?;

        if !temp_directory.ends_with('/') {
            temp_directory.push('/');
        }

        Ok(Config {
            directory: ConfigDirectory {
                source: source_directory,
                temp: temp_directory,
            },
            ftp: ConfigFtp {
                server: ftp_server,
                username: ftp_username,
                password: ftp_password,
            },
        })
    }
}
