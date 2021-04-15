use std::env;
use clap;
use std::collections::HashMap;

mod cli;
mod search;

fn main() {
    let yaml = clap::load_yaml!("cli.yaml");
    let matches = clap::App::from(yaml).get_matches();
        
    let result: Vec<String> = env::args().collect();
    let path = matches.value_of("path");
    let path =  match path{
        Some(path) => {path},
        None => {"./"}
    };

    cli::process_command(path, result);
}
