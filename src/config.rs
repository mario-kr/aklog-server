//! Module for parsing/accessing the configuration

extern crate log;
extern crate toml;

use std::path::PathBuf;
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

    /// Tries to open, read and parse a toml-file
    pub fn load(path: PathBuf) -> Result<ConfigDeser> {
        debug!("configuration file name: '{}'", path.display());

        let s = std::fs::read_to_string(&path)
            .chain_err(|| "configuration file could not be read")?;

        toml::from_str(&s)
            .map(|obj| {
                info!("successfully parsed configuration file");
                debug!("Config = {:?}", obj);
                obj
            })
            .map_err(|_| ErrorKind::ConfigParseError(path).into())
    }

    fn get_items(&self) -> &Vec<LogItemDeser> {
        &self.item
    }
}

//------------------------------------//
//  struct to access data later on    //
//------------------------------------//

/// The deserialized Item would nearly always require some operation on its
/// contents to use it, so we do those operations beforehand and only access
/// the useful data from main().
pub struct LogItem {
    file : String,
    regex : Regex,
    alias : String,
    capture_names : Vec<String>,
    aliases : Vec<String>,
}

impl LogItem {

    /// Transforms a LogItemDeser into a more immediately usable LogItem
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

        // The metric seen by grafana will be `alias.capturegroup_name`
        // One Regex may contain multiple named capture groups, so a vector
        // with all names is prepared here.
        let mut als : Vec<String> = Vec::new();
        for name in cnames.clone() {
            let mut temp = String::from(lid.alias.as_str());
            temp.push('.');
            temp.push_str(name.as_str());
            als.push(temp);
        }
        debug!("aliases: {:?}", als);

        Ok(
            LogItem {
                file : lid.file,
                regex : l_regex,
                alias: lid.alias,
                capture_names : cnames,
                aliases : als
            }
        )
    }

    pub fn file(&self) -> &String {
        &self.file
    }

    pub fn regex(&self) -> &Regex {
        &self.regex
    }

    pub fn alias(&self) -> &String {
        &self.alias
    }

    pub fn capture_names(&self) -> &Vec<String> {
        &self.capture_names
    }

    pub fn aliases(&self) -> &Vec<String> {
        &self.aliases
    }
}

/// Contains more immediately usable data
pub struct Config {
    items : Vec<LogItem>,
    all_aliases : Vec<String>,
}

impl Config {

    /// Lets serde do the deserialization, and transforms the given data
    /// for later access
    pub fn load(path: PathBuf) -> Result<Self> {
        let conf_deser = ConfigDeser::load(path)?;

        let mut l_items : Vec<LogItem> = Vec::new();
        for lid in conf_deser.get_items() {
            l_items.push(LogItem::from_log_item_deser((*lid).clone())?);
        }

        // combines all aliases into one Vec for the /search endpoint
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

