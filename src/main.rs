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

    //! grafana only needs a "200 Ok" on /

    "Hello there!"
}

#[post("/search", format = "application/json", data = "<data>")]
fn search(data : Json<Search>, config: State<Config>) -> Json<SearchResponse> {

    //! /search is used to query what metrics are offered.
    //! In this case, those are the `alias.capturegroup_name` configured by
    //! the user of this programm.

    debug!("handling search request: {:?}", data.0);
    Json(
        SearchResponse(
            (*config.all_aliases()).clone()
        )
    )
}

#[post("/query", format = "application/json", data = "<data>")]
fn query(data: Json<Query>, config: State<Config>) -> Result<Json<QueryResponse>> {

    //! /query needs to return actual data (if available).
    //! the required metrics are sent by grafana in the `targets` field, as
    //! well as is the wanted timerange.
    //! The only sort of response written here is a `Series`, basically an
    //! Array/Vector of two float-values, the second being a timestamp.
    //! Returning a table is not implemented.

    debug!("handling query: {:?}", data.0);

    let targets = data.0.targets;
    debug!("targets: {:?}", targets);

    // create hashmap to iterate over
    let mut target_hash = hash_map_targets(&config, targets)?;

    let date_from = data.0.range.from.timestamp();
    let date_to = data.0.range.to.timestamp();

    let mut response : Vec<TargetData> = Vec::new();

    // iterate the HashMap
    for (_alias, &(logitem, ref cns)) in target_hash.iter() {

        // prepare an empty Vector of Series
        let mut series_vec = Vec::new();
        for &(_, ref t) in cns.iter() {
            series_vec.push(Series{ target : (*t).clone(), datapoints : Vec::new() });
        }

        // open the current file for reading
        let mut line_iter = BufReader::new(
            File::open(logitem.file())
            .chain_err(|| format!("antikoerper log file could not be opened: {}", logitem.file()))?
            ).lines();

        // read the file line by line...
        while let Some(Ok(line)) = line_iter.next() {

            // ...and apply the configured regex to it.
            if let Some(capture_groups) = logitem.regex().captures_iter(&line).next() {

                // save the timestamp for later
                let timestamp = capture_groups["ts"]
                    .parse::<f64>()
                    .chain_err(|| "Failed to parse the filestamp")?;

                // ignore every entry not in the timerange
                if (timestamp as i64) > date_from && (timestamp as i64) < date_to {

                    // Multiple Vectors need to be accessed with the same
                    // index, so no iterator here.
                    for i in 0..cns.len() {

                        // get the current metric and parse its content as a
                        // float
                        let captured = capture_groups[
                            cns.get(i)
                                .ok_or(Error::from("out of bounds: capture_groups"))?
                                .0.as_str()
                        ].parse::<f64>()
                        .chain_err(|| "failed to parse the capture group")?;

                        // put the current metric and timestamp into the right
                        // Series
                        series_vec
                            .get_mut(i)
                            .ok_or(Error::from("out of bounds: series_vec"))?
                            .datapoints
                            .push([
                                  captured,
                                  // grafana requires ms
                                  timestamp * 1000.0
                            ]);
                    }
                }
            }
        }

        // fill the prepared vector with all Series's
        for series in series_vec.iter() {
            response.push(TargetData::Series((*series).clone()));
        }
    }

    Ok( Json( QueryResponse{ 0 : response } ) )
}

/// If there are several targets, it is possible they would different data
/// from the same file;
/// this HashMap is created for the sole purpose of being able to read and
/// apply a regex on a potentially huge file only once.
/// HashMap
/// |------- Alias : &String
/// \
///  Tuple
///  |------- &LogItem
///  |------- Vector of Tuple
///           |--- capturegroup name : String
///           |--- target/metric
fn hash_map_targets<'a>(c : &'a Config, targets : Vec<Target>)
    -> Result<HashMap<&'a String, (&'a LogItem, Vec<(String, String)>)>> {

    let mut _res : HashMap<&String, (&LogItem, Vec<(String, String)>)> = HashMap::new();
    for li in c.items() {
        for t in targets.clone() {
            if li.aliases().contains(&t.target) {
                if _res.contains_key(&li.alias()) {
                    if let Some(&mut (_litem, ref mut cnames)) = _res.get_mut(&li.alias()) {
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
                    _res.insert(
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
    Ok(_res)
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

    // Set level of verbosity and initialize the logger
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
