use std::env;

mod cli;
mod search;

fn main() {
    let yaml = clap::load_yaml!("cli.yaml");
    let matches = clap::App::from(yaml).get_matches();
        
    let result: Vec<String> = env::args().collect();
    let patterns = matches.values_of("path");

    if let Some(patterns) = patterns{
        for pattern in patterns {
            cli::process_command(pattern, result.iter(), &matches);
        }
    }
    else {
        cli::process_command("./*", result.iter(), &matches);
    }
    println!();
}
