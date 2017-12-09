//! Module for parsing/accessing the configuration

extern crate log;
extern crate toml;

use std::fs::File;
use std::io::Read;
use error::*;

/// Main Config struct
#[derive(Debug, Deserialize)]
pub struct Config {
    item : Vec<Item>,
}

/// Holds data for one antikoerper Log-File
#[derive(Debug, Deserialize)]
struct Item {
    file : String,
    regex : String,
    alias : String,
}

impl Config {
    pub fn load(filename : String) -> Result<Config> {
        debug!("configuration file name: '{}'", filename);
        let mut file = File::open(filename.clone())
            .chain_err(|| "configuration file could not be opened")?;
        debug!("configuration file successfully opened");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .chain_err(|| "configuration file could not be read")?;
        debug!("configuration file successfully read");
        match toml::from_str(content.as_str()) {
            Ok(config) => {
                debug!("configuration file successfully parsed");
                Ok(config)
            },
            _ => Err(ErrorKind::ConfigParseError(filename).into()),
        }
    }
}
