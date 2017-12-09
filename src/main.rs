#![recursion_limit = "1024"]

extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate simplelog;

use clap::{App, Arg};
use simplelog::{SimpleLogger, LogLevelFilter, Config as LogConfig};

use std::process::exit;

mod error;
mod config;
use config::Config;

fn main() {
    let matches = App::new("aklog-server")
                        .version("0.1.0")
                        .author("Mario Krehl <mario-krehl@gmx.de>")
                        .about("Presents antikoerper-logfiles to grafana")
                        .arg(Arg::with_name("config")
                             .short("c")
                             .long("config")
                             .value_name("FILE")
                             .help("configuration file to use")
                             .takes_value(true)
                             .required(true))
                        .arg(Arg::with_name("verbosity")
                             .short("v")
                             .long("verbose")
                             .help("sets the level of verbosity")
                             .multiple(true))
                        .get_matches();

    match matches.occurrences_of("verbosity") {
        0 => SimpleLogger::init(LogLevelFilter::Warn, LogConfig::default()).unwrap(),
        1 => SimpleLogger::init(LogLevelFilter::Info, LogConfig::default()).unwrap(),
        2 => SimpleLogger::init(LogLevelFilter::Debug, LogConfig::default()).unwrap(),
        3 | _  => SimpleLogger::init(LogLevelFilter::Trace, LogConfig::default()).unwrap(),
    };

    debug!("Initialized logger");

    let config_file = matches.value_of("config").unwrap();
    let config = match Config::load(String::from(config_file)) {
        Ok(c) => c,
        Err(_) => {
            error!("Failed to open/parse configuration file: '{}'", config_file);
            exit(1);
        },
    };

}
