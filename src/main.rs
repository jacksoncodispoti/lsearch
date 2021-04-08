use std::env;
use clap::{Arg, App};

mod cli;
mod search;

fn main() {
    let matches = App::new("L-Search")
        .version("0.0.1")
        .author("Alerik <alerik@alerik.de>")
        .about("Search through ALL files")
        .arg(Arg::new("path")
             .short('p')
             .long("path")
             .takes_value(true)
             .value_name("PATH"))

        //Filters
        .arg(Arg::new("is")
             .short('i')
             .long("is")
             .multiple(true)
             .about("Filter to match on value")
             .takes_value(true))
        .arg(Arg::new("not")
             .short('n')
             .long("not")
             .multiple(true)
             .about("Filter to not match on value")
             .takes_value(true))
        .arg(Arg::new("has")
             .short('h')
             .long("has")
             .multiple(true)
             .about("Filter to include on matches"))
        .arg(Arg::new("hasnt")
             .short('H')
             .long("hasnt")
             .multiple(true)
             .about("Filter to include on matches")
             .takes_value(true))
        
        //Scorers
        .arg(Arg::new("more")
             .short('m')
             .long("more")
             .multiple(true)
             .about("A scorer to increase based on matches")
             .takes_value(true))
        .arg(Arg::new("less")
             .short('l')
             .long("less")
             .multiple(true)
             .about("A scorer to decrease matches")
             .takes_value(true))
        
        //Content types
        .arg(Arg::new("content-text")
             .long("content-text")
             .about("Use file contents"))
        .arg(Arg::new("content-ext")
             .long("content-ext")
             .about("Use file extension"))
        .arg(Arg::new("content-title")
             .long("content-title")
             .about("Use file title"))
        .arg(Arg::new("content-path")
             .long("content-path")
             .about("Use file path"))
        //.arg(Arg::new("content-exec")
        //     .long("content-exec")
        //     .about("Use file exec"))
        .get_matches();

    let result: Vec<String> = env::args().collect();
    let path = matches.value_of("path");
    let path =  match path{
        Some(path) => {path},
        None => {"./"}
    };

    cli::process_command(path, result);
}
