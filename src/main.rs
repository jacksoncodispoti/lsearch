use clap::{App};
use std::path::Path;
use lsearch::search;

fn main() {
    let _matches = App::new("L-Search")
        .version("0.0.1")
        .author("Alerik <alerik@alerik.de>")
        .about("Search through ALL files");

    let path = Path::new("./");
    //let spec = search::DirScoreSpec{is_dir:false};
    //let spec = search::PermissionScoreSpec{permission: 0o777};
    let spec = search::PermissionScoreSpec{permission: 0o644};
    let scored_files = search::metadata_search(&path, &spec);

    for file in scored_files {
        let path = file.entry.path().into_os_string().into_string().unwrap();
        println!("{} [{}]", path, file.score);
    }
}
