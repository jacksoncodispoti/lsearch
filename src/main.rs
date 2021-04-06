use std::env;
use lsearch::cli;

fn main() {
   // let matches = App::new("L-Search")
   //     .version("0.0.1")
   //     .author("Alerik <alerik@alerik.de>")
   //     .about("Search through ALL files")
   //     .arg(Arg::new("more")
   //      .short('m')
   //      .long("more")
   //      .multiple(true)
   //      .about("A scorer to increase based on word count")
   //      .takes_value(true))
   //     .arg(Arg::new("less")
   //      .short('l')
   //      .long("less")
   //      .multiple(true)
   //      .about("A scorer to decrease based on word count")
   //      .takes_value(true))
   //     .arg(Arg::new("than")
   //      .short('t')
   //      .long("than")
   //      .multiple(true)
   //      .about("Use in conjunction with --more and --less")
   //      .takes_value(true));
        //.get_matches();

    let result: Vec<String> = env::args().collect();

    cli::process_command(result);
}
