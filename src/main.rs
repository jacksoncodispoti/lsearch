use std::env;
use clap;

mod cli;
mod search;

fn main() {
    let yaml = clap::load_yaml!("cli.yaml");
    let matches = clap::App::from(yaml).get_matches();
        
    let result: Vec<String> = env::args().collect();
    let paths = matches.values_of("path");

    if let Some(paths) = paths{
        for path in paths {
            cli::process_command(path, result.iter(), &matches);
        }
    }
    else {
        cli::process_command("./", result.iter(), &matches);
    }
    println!();
}
