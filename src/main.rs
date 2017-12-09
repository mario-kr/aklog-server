#![recursion_limit = "1024"]

extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate simplelog;

use std::process::exit;
use std::fs::File;

use clap::{App, Arg};
use simplelog::{SimpleLogger, LogLevelFilter, Config as LogConfig};

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
        Err(e) => {
            error!("{}", e);
            exit(1);
        },
    };
    let items = config.items();

    let first_item = items.first().unwrap();

    let file = File::open(first_item.file()).unwrap();
    use std::io::BufReader;
    use std::io::BufRead;
    let bufreader = BufReader::new(file);
    let mut capturename_iter = first_item.regex().capture_names().skip(1);
    while let Some(Some(name)) = capturename_iter.next() {
        println!("Named Capture: {}", name);
    }
    let mut line_iter = bufreader.lines();
    while let Some(Ok(line)) = line_iter.next() {
        if first_item.regex().is_match(line.as_str()) {
            println!("{}", line);
        }
        else {
            println!("did not match");
        }
    }
}
