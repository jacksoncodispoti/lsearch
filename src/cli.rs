use std::collections::HashMap;
use crate::search;
use walkdir::WalkDir;
use std::path;

mod stats {
    use std::collections::HashMap;
    #[derive(Debug)]
    pub struct RunStats {
        operations: HashMap<String, u32>,
        avg_length: f32,
        n: usize
    }

    impl RunStats {
        pub fn add_operation(&mut self, operation: String) {
            if self.operations.contains_key(&operation) {
                let next = self.operations.get(&operation).unwrap() + 1;
                self.operations.insert(operation, next);
            }
            else{
                self.operations.insert(operation, 1);
            }
        }
        pub fn add_length(&mut self, length: usize) {
            self.avg_length = (self.n as f32 * self.avg_length as f32 + length as f32) / (self.n as f32 + 1.0);
            self.n += 1;
        }

        pub fn new() -> RunStats {
            RunStats {
                operations: HashMap::new(), 
                avg_length: 0.0, 
                n: 0
            }
        }
    }
}

use std::fs;

fn get_content_loaders(loaders: std::slice::Iter<String>) -> HashMap<String, Box<dyn search::loaders::ContentLoader>> {
    let mut content_loaders: HashMap<String, Box<dyn search::loaders::ContentLoader>> = HashMap::new();

    for loader in loaders {
        //Path based
        if !content_loaders.contains_key(&String::from(loader)) {
            content_loaders.insert(String::from(loader), 
                                   match loader.as_str() {
                                       "--content-path" => { Box::new(search::loaders::PathLoader::new()) },
                                       "--content-title" => {Box::new(search::loaders::TitleLoader::new())},
                                       "--content-ext" => {Box::new(search::loaders::ExtLoader::new())},
                                       _ =>  {Box::new(search::loaders::TextLoader::new())}
                                   });
        }
    }

    content_loaders
}

struct ContentRun {
    content_loader: String,
    scorers: Vec<Box<dyn search::scorers::ContentScorer>>,
    targets: Vec<String>,
    insensitive: bool
}

impl ContentRun {
    fn is_valid(&self) -> bool {
        let mut legit = false;

        for scorer in &self.scorers {
            if scorer.get_name() != String::from("Pass") {
                legit = true;
                break;
            }
        }

        legit
    }
}

fn get_content_runs(args: std::slice::Iter<String>) -> Vec<ContentRun> {
    let mut current_loader = "--content-text";
    let mut current_run: ContentRun = ContentRun{content_loader: String::from(current_loader), scorers: Vec::new(), targets: Vec::new(), insensitive: true};
    let mut content_runs: Vec<ContentRun> = Vec::new();

    let mut scorer: Box<dyn search::scorers::ContentScorer> = Box::new(search::scorers::Pass{});

    let mut insensitive = false;
    for arg in args {
        if arg.starts_with("--") {
            //Content loading
            if arg.starts_with("--insensitive") {
                insensitive = true;
            }
            if arg.starts_with("--content") {
                current_loader = arg;

                if current_run.is_valid() {
                    content_runs.push(current_run);
                }

                current_run = ContentRun{content_loader: String::from(current_loader), scorers: Vec::new(), targets: Vec::new(), insensitive: insensitive};
                continue;
            }

            match arg.as_str() {
                //Filter/Scorer
                "--is" => { current_run.scorers.push(Box::new(search::scorers::Is{}))},
                "--not" =>{ current_run.scorers.push(Box::new(search::scorers::Not{}))},
                "--has" =>{ current_run.scorers.push(Box::new(search::scorers::Has{}))},
                "--hasnt" =>{ current_run.scorers.push(Box::new(search::scorers::Hasnt{}))},
                "--more" =>{ current_run.scorers.push(Box::new(search::scorers::More{}))},
                _ =>{ }
            };

            continue;
        }

        current_run.targets.push(String::from(arg));
    }
    if current_run.is_valid() {
        content_runs.push(current_run);
    }

    return content_runs;
}

//For optimizing later
//fn optimize_content_run_order(&mut Vec<ContentRun> runs) {
//
//}
//
fn summarize_runs(runs: std::slice::Iter<ContentRun>) {
    println!("Summarizing Operational Runs:");
    let mut count: u32 = 0;
    for run in runs {
        println!("Content: {}", run.content_loader);

        for (scorer, target) in run.scorers.iter().zip(run.targets.iter()) {
            println!("\t{}({})", scorer.get_name(), target);
        }
        count += 1;
    }

    if count == 0 {
        println!("\tNo operational runs");
    }
}

struct FileTraverseSpecs {
    recursive: bool,
    hidden: bool
}

impl FileTraverseSpecs {
    fn new(recursive: bool, hidden:bool) -> FileTraverseSpecs {
        FileTraverseSpecs{recursive: recursive,  hidden: hidden}
    }
}

fn get_file_traverse_specs(args: std::slice::Iter<String>) -> FileTraverseSpecs {
    let mut recursive = false;
    let mut hidden = false;

    for arg in args {
        match arg.as_str() {
            "--recursive" => { recursive = true; },
            "--hidden" => { hidden = true; },
            _ => {}
        }
    }

    return FileTraverseSpecs::new(recursive, hidden);
}

pub fn process_command(path: &str, args: Vec<String>) -> u32 {
    let mut path = path::PathBuf::from(path);
    println!("\tls {:?}", path);
    path = fs::canonicalize(&path).unwrap();
    println!("\tls {:?}", path);
    //let command_order = process_command_order(args);
    let runs = get_content_runs(args.iter());
    let traverseSpecs = get_file_traverse_specs(args.iter());
    let loader_names: Vec<String> = runs.iter().map(|r| String::from(&r.content_loader)).collect();
    let content_loaders = get_content_loaders(loader_names.iter());
    //
    //optimize_content_run_order(&mut runs);

    let mut next_directories: Vec<(f32, walkdir::DirEntry)> = Vec::new();
    let mut content_run_stats: Vec<stats::RunStats> = Vec::new();


    if args.contains(&String::from("--strats")) {
        summarize_runs(runs.iter());
    }

    for run in runs {
        let mut run_stats = stats::RunStats::new();
        let directories = WalkDir::new(&path);

        for direntry in directories {
            let direntry = direntry.unwrap();
            let content_loader = content_loaders.get(&run.content_loader).expect("Unable to get content loader");
            let mut content = content_loader.load_content(&direntry);
           
            if args.contains(&String::from("--insensitive")) {
                content = content.to_ascii_lowercase();
            }

            let mut filtered = true;
            let mut score = 0.0;
            for (scorer, target) in run.scorers.iter().zip(run.targets.iter()) {
                run_stats.add_operation(scorer.get_name());
                let target = target.to_ascii_lowercase();
                let ind_score = scorer.score(&content, &target);
                //println!("\t{:?}", scorer);
                if content.len() < 40{
                }
                score += ind_score; 

                if ind_score < 1.0 {
                    filtered = false;
                    break;
                }
            }
            run_stats.add_length(content.len());
            if filtered {
                next_directories.push((score, direntry));
            }
        }
        next_directories.sort_by(|a,b| a.0.partial_cmp(&b.0).unwrap());
        next_directories.reverse();
        //directories = next_directories.into_iter().collect();
        content_run_stats.push(run_stats);
    }

    for (score, direntry) in next_directories {
        if args.contains(&String::from("--score")) {
            println!("[{}] {}", score, direntry.path().as_os_str().to_str().unwrap());
        }
        else{
            println!("{}", direntry.path().as_os_str().to_str().unwrap());
        }
    }

    if args.contains(&String::from("--stats")) {
        println!("{:?}", content_run_stats);
    }
    0
}
