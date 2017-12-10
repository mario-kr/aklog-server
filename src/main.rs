#![recursion_limit = "1024"]

#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate clap;
extern crate chrono;
extern crate dimensioned;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate regex;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate simplelog;

use std::process::exit;
use std::fs::File;

use clap::{App, Arg};
use chrono::prelude::*;
use rocket::State;
use rocket_contrib::Json;
use simplelog::{SimpleLogger, LogLevelFilter, Config as LogConfig};

mod api;
mod config;
mod error;
use api::*;
use config::Config;
use error::*;

#[get("/")]
fn index() -> &'static str {
    "Hello there!"
}

#[post("/search", format = "application/json", data = "<data>")]
fn search(data : Json<Search>, config: State<Config>) -> Json<SearchResponse> {
    debug!("handling search request: {:?}", data.0);
    Json(
        SearchResponse(
            *config.all_aliases()
        )
    )
}

#[post("/query", format = "application/json", data = "<data>")]
fn query(data: Json<Query>, config: State<Config>) -> Result<Json<QueryResponse>> {
    Err(Error::from("not implemented"))
}

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

    rocket::ignite()
        .manage(config)
        .mount("/", routes![index, search, query])
        .launch();
}
