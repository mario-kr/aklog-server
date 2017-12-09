//! Module for parsing/accessing the configuration

extern crate log;
extern crate toml;

use std::fs::File;
use std::io::Read;
use regex::Regex;
use error::*;

//------------------------------------//
//  structs for deserialization       //
//------------------------------------//

/// Holds data for one antikoerper Log-File.
/// Used for deserialization only
#[derive(Clone, Debug, Deserialize)]
pub struct LogItemDeser {
    file : String,
    regex : String,
    alias : String,
}

impl LogItemDeser {
    pub fn file(&self) -> String {
        self.file.clone()
    }
    pub fn regex(&self) -> Result<Regex> {
        debug!("trying to parse regex `{}`", self.regex);
        Regex::new(self.regex.as_str()).chain_err(|| format!("failed to parse regex `{}`", self.regex))
    }
    pub fn alias(&self) -> String {
        self.alias.clone()
    }
}

/// Used for deserialization only
#[derive(Debug, Deserialize)]
pub struct ConfigDeser {
    item : Vec<LogItemDeser>,
}

impl ConfigDeser {
    pub fn load(filename : String) -> Result<ConfigDeser> {
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
                info!("successfully parsed configuration file");
                Ok(config)
            },
            _ => Err(ErrorKind::ConfigParseError(filename).into()),
        }
    }

    pub fn get_items(&self) -> Vec<LogItemDeser> {
        self.item.clone()
    }
}

//------------------------------------//
//  struct to access data later on    //
//------------------------------------//

pub struct LogItem {
    file : String,
    regex : Regex,
    aliases : Vec<String>,
}

impl LogItem {
    fn from_log_item_deser(lid : LogItemDeser) -> Result<LogItem> {
        debug!("trying to parse regex `{}`", lid.regex);
        let l_regex = Regex::new(lid.regex.as_str())
            .chain_err(|| format!("regex not parseable: '{}'", lid.regex))?;
        let cnames : Vec<String> = l_regex
            .capture_names()
            .skip(2)
            .filter_map(|n| n)
            .map(|n| String::from(n))
            .collect();
        debug!("capture names: {:?}", cnames);
        let mut als : Vec<String> = Vec::new();
        for name in cnames {
            let mut temp = String::from(lid.alias.as_str());
            temp.push('.');
            temp.push_str(name.as_str());
            als.push(temp);
        }
        debug!("aliases: {:?}", als);

        Ok(LogItem { file : lid.file, regex : l_regex, aliases : als })
    }
}

pub struct Config {
    items : Vec<LogItem>,
    all_aliases : Vec<String>,
}

impl Config {
    pub fn load(filename : String) -> Result<Self> {
        Err(ErrorKind::ConfigParseError(String::from("meh")).into())
    }
}

