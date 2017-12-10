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
extern crate serde_json;
extern crate simplelog;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::process::exit;

use clap::{App, Arg};
use rocket::State;
use rocket_contrib::Json;
use simplelog::{SimpleLogger, LogLevelFilter, Config as LogConfig};

mod api;
mod config;
mod error;
use api::*;
use config::{Config, LogItem};
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
            (*config.all_aliases()).clone()
        )
    )
}

#[post("/query", format = "application/json", data = "<data>")]
fn query(data: Json<Query>, config: State<Config>) -> Result<Json<QueryResponse>> {
    debug!("handling query: {:?}", data.0);
    let targets = data.0.targets;
    debug!("targets: {:?}", targets);
    let mut target_hash : HashMap<&String, (&LogItem, Vec<(String, String)>)> = HashMap::new();
    for li in config.items() {
        for t in targets.clone() {
            if li.aliases().contains(&t.target) {
                if target_hash.contains_key(&li.alias()) {
                    if let Some(&mut (_litem, ref mut cnames)) = target_hash.get_mut(&li.alias()) {
                        cnames.push((
                                t.target
                                    .split('.')
                                    .nth(1)
                                    .ok_or(Error::from("no capture name found"))?
                                    .into(),
                                t.target.clone())
                            );
                    }
                }
                else {
                    target_hash.insert(
                        li.alias(),
                        (&li, vec![(
                            t.target
                                .split('.')
                                .nth(1)
                                .ok_or(Error::from("no capture name found"))?
                                .into(),
                            t.target.clone())
                            ]
                        )
                    );
                }
            }
        }
    }

    let mut response : Vec<TargetData> = Vec::new();
    for (_alias, &(logitem, ref cns)) in target_hash.iter() {
        let mut series_vec = Vec::new();
        for &(_, ref t) in cns.iter() {
            series_vec.push(Series{ target : (*t).clone(), datapoints : Vec::new() });
        }
        let mut line_iter = BufReader::new(
            File::open(logitem.file())
            .chain_err(|| format!("antikoerper log file could not be opened: {}", logitem.file()))?
        ).lines();
        while let Some(Ok(line)) = line_iter.next() {
            let capture_groups = logitem
                .regex()
                .captures_iter(&line)
                .next()
                .ok_or(Error::from("regex did not match"))?;
            let timestamp = capture_groups["ts"]
                .parse::<f64>()
                .chain_err(|| "Failed to parse the filestamp")?;
            for i in 0..cns.len() {
                let captured = capture_groups[
                    cns.get(i)
                        .ok_or(Error::from("out of bounds: capture_groups"))?
                        .0.as_str()
                ].parse::<f64>()
                .chain_err(|| "failed to parse the capture group")?;
                series_vec
                    .get_mut(i)
                    .ok_or(Error::from("out of bounds: series_vec"))?
                    .datapoints
                    .push([
                          captured,
                          timestamp
                    ]);
            }
        }
        for series in series_vec.iter() {
            response.push(TargetData::Series((*series).clone()));
        }
    }

    Ok( Json( QueryResponse{ 0 : response } ) )
        /*    Series{
                target : *k,
            BufReader::new(File::open(li.file())?).lines()
                .map(|line| {
                    //let capture_groups = li.regex().captures_iter(line).first()?;

    Err(Error::from("not implemented"))
    */
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
