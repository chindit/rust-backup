# Rust-backup

In order to run, please create a config file in `/etc/plex-backup.json`

Config content:

```json
{
    "sourceDirectory":"/path/to/directory/to/backup",
    "ftp": {
        "server":"ftp.server.com:21",
        "username":"username",
        "password":"password"
    }
}
```
