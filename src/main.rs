#![recursion_limit = "1024"]

#![feature(decl_macro)]

extern crate clap;
extern crate chrono;
extern crate dimensioned;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate regex;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_regex;
extern crate simplelog;
extern crate getset;

use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::io::{BufReader, BufRead};
use std::str::FromStr;

use clap::{App, Arg};
use rocket::State;
use rocket_contrib::json::Json;
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

    Ok(
        Json(
            QueryResponse{
                0 : hash_map_iter(
                        hash_map_targets(&config, data.0.targets)?,
                        data.0.range.from.timestamp(),
                        data.0.range.to.timestamp()
                    )?
            }
        )
    )
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

    debug!("targets: {:?}", targets);
    let mut res : HashMap<&String, (&LogItem, Vec<(String, String)>)> = HashMap::new();
    for li in c.items() {
        for t in targets.iter() {
            if li.aliases().contains(&t.target) {
                if res.contains_key(&li.file()) {
                    if let Some(&mut (_litem, ref mut cnames)) = res.get_mut(&li.file()) {
                        cnames.push((
                                cname_from_target(&t.target)?,
                                t.target.clone())
                        );
                    }
                }
                else {
                    res.insert(
                        li.file(),
                        (
                            &li,
                            vec![(cname_from_target(&t.target)?, t.target.clone())]
                        )
                    );
                }
            }
        }
    }

    Ok(res)
}

/// splits the target and return the capture name part
fn cname_from_target<'a>(t : &'a String) -> Result<String> {
    t.split('.').nth(1).map(str::to_string).ok_or(Error::from("no capture name found").into())
}

/// Iterate the hashmap created with the above function
fn hash_map_iter(h : HashMap<&String, (&LogItem, Vec<(String, String)>)>, d_from : i64, d_to : i64)
    -> Result<Vec<TargetData>> {

    let mut res = Vec::new();
    for (file, &(logitem, ref cns)) in h.iter() {

        // prepare an empty Vector of Series
        let mut series_vec = cns.iter().map(|tpl| tpl.1.to_string())
            .map(|target| Series { target, datapoints: Vec::new() })
            .collect::<Vec<_>>();

        // open the current file for reading
        let mut line_iter = BufReader::new(
            File::open(file)
            .chain_err(|| format!("log file could not be opened: {}", logitem.file()))?
            ).lines();

        // read the file line by line...
        while let Some(Ok(line)) = line_iter.next() {

            // ...and apply the configured regex to it.
            if let Some(capture_groups) = logitem.regex().captures_iter(&line).next() {

                // save the timestamp for later
                let timestamp = capture_groups["ts"]
                    .parse::<f64>()
                    .chain_err(|| "Failed to parse the timestamp")?;

                // ignore every entry not in the timerange
                if (timestamp as i64) > d_from && (timestamp as i64) < d_to {

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
        res.extend(series_vec.into_iter().map(TargetData::Series));
    }
    Ok(res)
}


fn main() -> Result<()> {

    let matches = App::new("aklog-server")
        .version("0.1.0")
        .author("Mario Krehl <mario-krehl@gmx.de>")
        .about("Presents regex-parsable data to grafana")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("configuration file to use")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("address")
             .long("address")
             .value_name("ADDR")
             .help("Address to bind to")
             .takes_value(true)
             .default_value("127.0.0.1")
             .required(false))
        .arg(Arg::with_name("port")
             .long("port")
             .value_name("PORT")
             .help("Port to bind to")
             .takes_value(true)
             .default_value("8000")
             .required(false))
        .arg(Arg::with_name("verbosity")
             .short("v")
             .long("verbose")
             .help("sets the level of verbosity")
             .multiple(true))
        .get_matches();

    // Set level of verbosity and initialize the logger
    let filter = match matches.occurrences_of("verbosity") {
        0      => LogLevelFilter::Warn,
        1      => LogLevelFilter::Info,
        2      => LogLevelFilter::Debug,
        3 | _  => LogLevelFilter::Trace,
    };
    SimpleLogger::init(filter, LogConfig::default()).unwrap();
    debug!("Initialized logger");

    let config_file = matches.value_of("config").unwrap();
    let config = Config::load(PathBuf::from(String::from(config_file)))
        .map_err(|e| format!("{}", e))?;

    let host = matches.value_of("address").unwrap(); // safe by clap
    let port = matches.value_of("port").map(u16::from_str)
        .transpose()
        .map_err(|e| format!("Parsing port failed: {:?}", e))?
        .unwrap(); // safe by clap

    rocket::custom({
        let mut c = rocket::config::Config::production();
        c.set_address(host).map_err(|e| format!("Using host address failed: {}: {}", host, e))?;
        c.set_port(port);
        c
    })
    .manage(config)
    .mount("/", routes![index, search, query])
    .launch();
    Ok(())
}
