use std::collections::HashMap;
use crate::search;
use walkdir::WalkDir;
use search::scorers::fs::{DirEntryFilter, HiddenFilter};
use std::path;
use std::fs;


mod stats {
    use std::collections::HashMap;
    use std::time::Instant;
    use std::fmt;

    #[derive(Debug)]
    pub struct OperationStats {
        name: String,
        n: usize,
        avg_time: f32,
        avg_size: f32,
        instant: Instant
    }

    impl fmt::Display for OperationStats {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "\t\t{} [n={}, avg_t={:.1}ns, avg_s={:.1}]", self.name, self.n, self.avg_time, self.avg_size)
        }
    }

    impl OperationStats {
        fn new(name: &String) -> OperationStats {
            OperationStats { name: String::from(name), n: 0, avg_time: 0.0, avg_size: 0.0, instant: Instant::now() }
        }

        fn start(&mut self, content_len: usize) {
            self.avg_size = (self.n as f32 * self.avg_size + content_len as f32) / (self.n as f32 + 1.0);
            self.instant = Instant::now();
        }

        fn stop(&mut self) {
            let elapsed = self.instant.elapsed().as_nanos() as f32 / 1000.0;
            self.avg_time = (self.n as f32 * self.avg_time + elapsed) / (self.n as f32 + 1.0);
            self.n += 1;
        }
    }

    #[derive(Debug)]
    pub struct RunStats {
        operations: HashMap<String, OperationStats>,
        operation_order: Vec<String>,
        targets: Vec<String>,
        content_loader: String,
        avg_length: f32,
        n: usize,
        time: u128,
        instant: Instant
    }

    impl fmt::Display for RunStats {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "\t{} [t={}μs]", self.content_loader, self.time as f32 / 1000.0).unwrap();
            for (op, target) in self.operation_order.iter().zip(self.targets.iter()) {
                let key = crate::search::scorers::create_key(&op, &target);
                match self.operations.get(&key) {
                    Some(operation) => { write!(f, "{}", operation).unwrap(); },
                    None => { writeln!(f, "\t\t{} (Never executed)", op).unwrap(); }
                }
            }
            Ok(())
        }
    }

    impl RunStats {
        pub fn add_length(&mut self, length: usize) {
            self.avg_length = (self.n as f32 * self.avg_length as f32 + length as f32) / (self.n as f32 + 1.0);
            self.n += 1;
        }
        pub fn start_timer(&mut self) {
            self.instant = Instant::now();
        }
        pub fn stop_timer(&mut self) {
            let elapsed = self.instant.elapsed();
            self.time += elapsed.as_nanos();
        }
        pub fn start_operation(&mut self, operation: &String, content_len: usize) {
            if self.operations.contains_key(operation) {
                self.operations.get_mut(operation).expect("this should not happen").start(content_len);
            }
            else {
                self.operations.insert(String::from(operation), OperationStats::new(&operation));
            }
        }
        pub fn stop_operation(&mut self, operation: &String) {
            if self.operations.contains_key(operation) {
                self.operations.get_mut(operation).expect("This should not happen").stop();
            }
        }

        pub fn new(run: &crate::cli::ContentRun) -> RunStats {
            let operation_order: Vec<String> = run.scorers.iter().map(|s| s.get_name()).collect();
            let targets: Vec<String> = run.targets.iter().map(|s| String::from(s)).collect();

            RunStats {
                operations: HashMap::new(), 
                operation_order: operation_order,
                targets: targets,
                content_loader: String::from(&run.content_loader),
                avg_length: 0.0, 
                n: 0,
                time: 0,
                instant: Instant::now()
            }
        }
    }

    #[derive(Debug)]
    pub struct AppStats {
        runs: Vec<RunStats> 
    }

    impl fmt::Display for AppStats {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "Operational Statistics").unwrap();
            for run_stats in &self.runs {
                writeln!(f, "{}", run_stats).unwrap();
            }
            Ok(())
        }
    }

    impl AppStats {
        pub fn new() -> AppStats {
            AppStats { runs: vec![] }
        }

        pub fn push_run(&mut self, run_stats: RunStats) {
            self.runs.push(run_stats); 
        }
    }
}

pub struct ContentRun {
    content_loader: String,
    scorers: Vec<Box<dyn search::scorers::ContentScorer>>,
    targets: Vec<String>,
    insensitive: bool
}

fn get_content_loaders(loaders: std::slice::Iter<String>) -> HashMap<String, Box<dyn search::loaders::ContentLoader>> {
    let mut content_loaders: HashMap<String, Box<dyn search::loaders::ContentLoader>> = HashMap::new();

    for loader in loaders {
        //Path based
        if !content_loaders.contains_key(&String::from(loader)) {
            content_loaders.insert(String::from(loader), 
                                   match loader.as_str() {
                                       "--content-path" => { Box::new(search::loaders::PathLoader::new()) },
                                       "--content-path" => { Box::new(search::loaders::PathLoader::new()) },
                                       "--content-title" => {Box::new(search::loaders::TitleLoader::new())},
                                       "--content-ext" => {Box::new(search::loaders::ExtLoader::new())},
                                       "-E" => {Box::new(search::loaders::ExtLoader::new())},
                                       _ =>  {Box::new(search::loaders::TitleLoader::new())}
                                   });
        }
    }

    content_loaders
}


impl ContentRun {
    fn default() -> ContentRun {
        ContentRun { content_loader: String::from("--content-title"), scorers: vec![Box::new(search::scorers::Pass{})], targets: vec![String::from("")], insensitive: true }
    }

    fn new(content_loader: String, insensitive: bool) -> ContentRun {
        ContentRun { content_loader: content_loader, scorers: vec![], targets: vec![], insensitive: insensitive }
    }

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

fn is_is(s: &str) -> bool {
    true
}
fn get_content_runs(args: std::slice::Iter<String>, matches: &clap::ArgMatches) -> Vec<ContentRun> {
    let mut current_loader = "--content-title";
    let mut current_run: ContentRun = ContentRun{content_loader: String::from(current_loader), scorers: Vec::new(), targets: Vec::new(), insensitive: true};
    let mut content_runs: Vec<ContentRun> = Vec::new();

    let insensitive = false;
    for arg in args.skip(1) {
        if arg.starts_with("-") {
            //Content loading
            if arg == "--insensitive" {
                current_run.insensitive = true;
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
                "--content-ext" => { 
                    if current_run.is_valid() {
                        content_runs.push(current_run); 
                    }
                    current_loader = arg;

                    current_run = ContentRun::new(String::from(current_loader), insensitive);
                },
                "-E" => {
                    if current_run.is_valid() {
                        content_runs.push(current_run); 
                    }
                    current_loader = "--content-ext";

                    current_run = ContentRun::new(String::from(current_loader), insensitive);
                },
                "--content-path" => { 
                    if current_run.is_valid() {
                        content_runs.push(current_run); 
                    }
                    current_loader = arg;

                    current_run = ContentRun::new(String::from(current_loader), insensitive);
                },
                "-P" => {
                    if current_run.is_valid() {
                        content_runs.push(current_run); 
                    }
                    current_loader = "--content-path";

                    current_run = ContentRun::new(String::from(current_loader), insensitive);
                },
                "--content-text" => { 
                    if current_run.is_valid() {
                        content_runs.push(current_run); 
                    }
                    current_loader = arg;

                    current_run = ContentRun::new(String::from(current_loader), insensitive);
                },
                "-t" => {
                    if current_run.is_valid() {
                        content_runs.push(current_run); 
                    }
                    current_loader = "--content-text";

                    current_run = ContentRun::new(String::from(current_loader), insensitive);
                },
                "--content-title" => { 
                    if current_run.is_valid() {
                        content_runs.push(current_run); 
                    }
                    current_loader = arg;

                    current_run = ContentRun::new(String::from(current_loader), insensitive);
                },
                "-T" => {
                    if current_run.is_valid() {
                        content_runs.push(current_run); 
                    }
                    current_loader = "--content-title";

                    current_run = ContentRun::new(String::from(current_loader), insensitive);
                },
                //Filter/Scorer
                "--is" => { current_run.scorers.push(Box::new(search::scorers::Is{}))},
                "-e" => { current_run.scorers.push(Box::new(search::scorers::Is{}))},

                "--not" =>{ current_run.scorers.push(Box::new(search::scorers::Not{}))},
                "-n" =>{ current_run.scorers.push(Box::new(search::scorers::Not{}))},

                "--has" =>{ current_run.scorers.push(Box::new(search::scorers::Has{}))},
                "-h" =>{ current_run.scorers.push(Box::new(search::scorers::Has{}))},

                "--hasnt" =>{ current_run.scorers.push(Box::new(search::scorers::Hasnt{}))},
                "-H" =>{ current_run.scorers.push(Box::new(search::scorers::Hasnt{}))},

                "--more" =>{ current_run.scorers.push(Box::new(search::scorers::More{}))},
                "-m" =>{ current_run.scorers.push(Box::new(search::scorers::More{}))},
                _ =>{ }
            };

            continue;
        }

        current_run.targets.push(String::from(arg));
    }
    if current_run.is_valid() {
        content_runs.push(current_run);
    }

    if content_runs.len() == 0 {
        content_runs.push(ContentRun::default());
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
        println!("{} [insensitive={}]", run.content_loader, run.insensitive);

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

struct OutputSpecs {
    absolute: bool
}

impl FileTraverseSpecs {
    fn new(recursive: bool, hidden:bool) -> FileTraverseSpecs {
        FileTraverseSpecs{recursive: recursive,  hidden: hidden}
    }
}

impl OutputSpecs {
    fn new(absolute: bool) -> OutputSpecs {
        OutputSpecs{ absolute: absolute }
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

fn get_output_specs(args: std::slice::Iter<String>, matches: &clap::ArgMatches) -> OutputSpecs {
    let absolute = matches.is_present("absolute");

    return OutputSpecs::new(absolute);
}

fn get_content(run: &ContentRun, content_loaders: &HashMap<String, Box<dyn search::loaders::ContentLoader>>, direntry: &walkdir::DirEntry) -> String {
    let content_loader = content_loaders.get(&run.content_loader).expect("Unable to get content loader");
    let mut content = content_loader.load_content(&direntry);
   
    if run.insensitive {
        content = content.to_ascii_lowercase();
    }

    content
}

fn break_args(args: std::slice::Iter<String>) -> Vec<String> {
    let mut new_args: Vec<String> = vec![];
    for arg in args.into_iter() {
        if arg.starts_with("-") && arg.len() >= 2 && arg.as_bytes()[1] != '-' as u8 {
           let split: Vec<String> = arg.as_bytes().iter().skip(1)
               .map(|b| String::from("-") + &String::from(*b as char)).collect();
           new_args.extend(split);
        }
        else {
            new_args.push(arg.to_string());
        }
    }
    new_args
}


pub fn process_command(path: &str, args: Vec<String>, matches: &clap::ArgMatches) -> u32 {
    let mut path = path::PathBuf::from(path);
    let args = break_args(args.iter());
    //let command_order = process_command_order(args);
    let runs = get_content_runs(args.iter(), matches);

    let traverse_specs = get_file_traverse_specs(args.iter());
    let output_specs = get_output_specs(args.iter(), matches);
    let loader_names: Vec<String> = runs.iter().map(|r| String::from(&r.content_loader)).collect();
    let content_loaders = get_content_loaders(loader_names.iter());
    //
    //optimize_content_run_order(&mut runs);

    let mut app_stats = stats::AppStats::new();
    //let mut content_run_stats: Vec<stats::RunStats> = Vec::new();

    path = fs::canonicalize(&path).unwrap();
    if matches.is_present("echo") {
        println!("\tls {:?}", path);
        println!("\tls {:?}", path);
    }

    if matches.is_present("--strats") {
        summarize_runs(runs.iter());
    }

    let hidden_filter = HiddenFilter::new(traverse_specs.hidden);

    let directories = match traverse_specs.recursive {
        true => WalkDir::new(&path),
        false => WalkDir::new(&path).max_depth(1)
    }
    .sort_by(|a,b| b.file_name().cmp(a.file_name()));
    let mut directories: Vec<(f32, walkdir::DirEntry)> = directories.into_iter()
        .filter_map(|e| e.ok())
        .map(|a|(1.0, a))
        .collect();
    let mut next_directories: Vec<(f32, walkdir::DirEntry)>;

    for run in runs {
        let mut run_stats = stats::RunStats::new(&run);
        next_directories = Vec::new();

        for (_s, direntry) in directories.into_iter() {
            if !hidden_filter.filter(&direntry) {
                continue
            }

            let content = get_content(&run, &content_loaders, &direntry);
            let mut filtered = true;
            let mut score = 0.0;

            for (scorer, target) in run.scorers.iter().zip(run.targets.iter()) {
                let operation_key = search::scorers::create_key_from_scorer(&scorer, &target);
                let target = if run.insensitive { target.to_ascii_lowercase() } else { String::from(target) };

                run_stats.start_operation(&operation_key, content.len());
                let ind_score = scorer.score(&content, &target);
                run_stats.stop_operation(&operation_key);

                score += ind_score; 

                if ind_score < 1.0 {
                    filtered = false;
                    break;
                }
            }

            if filtered {
                next_directories.push((score, direntry));
            }
        }

        next_directories.sort_by(|a,b| a.0.partial_cmp(&b.0).unwrap());
        next_directories.reverse();
        run_stats.stop_timer();
        directories = next_directories;
        app_stats.push_run(run_stats);
    }
    
    let working_dir = std::env::current_dir().unwrap();

    for (score, direntry) in directories {
        let dir_path = direntry.path().as_os_str().to_str().unwrap();
        
        if output_specs.absolute{
            if matches.is_present("score") {
                println!("[{}] {}", score, dir_path);
            }
            else{
                println!("{}", dir_path);
            }
        }
        else{
            let clean_path = match dir_path.strip_prefix(working_dir.as_path().as_os_str().to_str().unwrap()) {
                Some(str) => if str.len() > 0 { &str[1..] } else { "" },
                None => ""
            };

            if matches.is_present("score") {
                println!("[{}] {}", score, clean_path);
            }
            else{
                println!("{}", clean_path);
            }
        }
    }

    if matches.is_present("stats") {
        print!("{}", app_stats);
    }
    0
}
