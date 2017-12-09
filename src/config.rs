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

    fn get_items(&self) -> &Vec<LogItemDeser> {
        &self.item
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
        // first capture is the whole match and nameless
        // second capture is always the timestamp
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

    pub fn file(&self) -> &String {
        &self.file
    }

    pub fn regex(&self) -> &Regex {
        &self.regex
    }

    pub fn aliases(&self) -> &Vec<String> {
        &self.aliases
    }
}

pub struct Config {
    items : Vec<LogItem>,
    all_aliases : Vec<String>,
}

impl Config {

    pub fn load(filename : String) -> Result<Self> {
        let conf_deser = ConfigDeser::load(filename)?;
        let mut l_items : Vec<LogItem> = Vec::new();
        for lid in conf_deser.get_items() {
            l_items.push(LogItem::from_log_item_deser((*lid).clone())?);
        }
        let mut all_als : Vec<String> = Vec::new();
        for li in &l_items {
            for als in li.aliases() {
                all_als.push((*als).clone());
            }
        }
        Ok(Config { items: l_items, all_aliases : all_als })
    }

    pub fn items(&self) -> &Vec<LogItem> {
        &self.items
    }

    pub fn all_aliases(&self) -> &Vec<String> {
        &self.all_aliases
    }
}

