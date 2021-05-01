use std::collections::HashMap;
use crate::search;
use walkdir::WalkDir;
use search::scorers::fs::{DirEntryFilter, HiddenFilter};
use std::path;
use std::fs;
use std::io::Write;
use colour;
use std::os::unix::fs::MetadataExt;
use bit_field::BitField;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::naive::NaiveDateTime;
use chrono::DateTime;


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
                content_loader: String::from(run.content_loader.get_name()),
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
    content_loader: Box<dyn search::loaders::ContentLoader>,
    scorers: Vec<Box<dyn search::scorers::ContentScorer>>,
    targets: Vec<String>,
    insensitive: bool
}

impl ContentRun {
    fn default() -> ContentRun {
        ContentRun { content_loader: Box::new(search::loaders::ContentTitle::new()), scorers: vec![Box::new(search::scorers::Pass{})], targets: vec![String::from("")], insensitive: true }
    }

    fn new(content_loader: Box<dyn search::loaders::ContentLoader>, insensitive: bool) -> ContentRun {
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

struct Arg {
    pub short: char,
    pub long: String,
    value: Option<String>
}

impl Arg {
    fn new(short: char, long: &str) -> Arg {
        Arg{short: short, long: String::from(long), value: None} 
    }

    fn is(&self, other: &str) -> bool {
        let first_char = other.chars().next().unwrap();
        if other.starts_with("--") {
            return self.long == &other[2..];
        }
        else if other.starts_with("-") {
            return self.short == first_char;
        }
        else {
            return self.long == other || if other.len() == 1 { self.short == first_char } else { false };
        }
    }

    fn set_value(&mut self, other: &str) {
        self.value = Some(String::from(other));
    }

    fn get_value(&self) -> Option<String> {
        match &self.value {
            Some(val) => Some(String::from(val)),
            None => None
        }
    }
}


fn parse_args(args: std::slice::Iter<String>) -> Vec<Arg> {
    let arg_lookup: Vec<(char, &str)> = vec![
        ('A', "absolute"),
        ('E', "content-ext"),
        ('P', "content-path"),
        ('t', "content-text"),
        ('T', "content-title"),
        ('C', "content-exec"),
        ('\0', "echo"),
        ('\0', "help"),
        ('a', "hidden"),
        ('i', "insensitive"),
        ('r', "recursive"),
        ('\0', "score"),
        ('\0', "stats"),
        ('\0', "strats"),
        ('V', "version"),
        ('h', "has"),
        ('H', "hasnt"),
        ('e', "is"),
        ('L', "less"),
        ('l', "long"),
        ('m', "more"),
        ('n', "not")
    ];

    let mut parsed_args: Vec<Arg> = vec![];

    for arg in args {
        if arg.starts_with("--") {
           let arg = arg_lookup.iter().find(|a|a.1==&arg[2..]).unwrap();
           parsed_args.push(Arg::new(arg.0, arg.1));
        }
        else if arg.starts_with("-") {
           let split: Vec<char> = arg.as_bytes().iter().skip(1)
               .map(|b| *b as char).collect();

           for sub_arg in split {
               let arg = arg_lookup.iter().find(|a|a.0==sub_arg).unwrap();
               parsed_args.push(Arg::new(arg.0, arg.1));
           }
        }
        else {
            match parsed_args.last_mut() {
                Some(parsed_arg) => { parsed_arg.set_value(arg); }
                None => {}
            };
        }
    }

    parsed_args
}

fn get_content_runs(args: std::slice::Iter<Arg>, _matches: &clap::ArgMatches) -> Vec<ContentRun> {
    let mut current_loader: Box<dyn search::loaders::ContentLoader> = Box::new(search::loaders::ContentTitle::new());
    let mut current_run: ContentRun = ContentRun{content_loader: current_loader, scorers: Vec::new(), targets: Vec::new(), insensitive: true};
    let mut content_runs: Vec<ContentRun> = Vec::new();

    let insensitive = false;
    for arg in args {
        if let Some(loader) = search::loaders::parse(&arg.long) {
            current_loader = loader;

            if current_run.is_valid() {
                content_runs.push(current_run);
            }

            current_run = ContentRun{content_loader: current_loader, scorers: Vec::new(), targets: Vec::new(), insensitive: insensitive};
            continue;
        }
        else if arg.is("content-exec") {
            current_loader = Box::new(search::loaders::ContentExec::new(&arg.get_value().unwrap()));

            if current_run.is_valid() {
                content_runs.push(current_run);
            }

            current_run = ContentRun{content_loader: current_loader, scorers: Vec::new(), targets: Vec::new(), insensitive: insensitive};
            continue;
        }
        else if arg.is("insensitive") {
                current_run.insensitive = true;
        }
        else if arg.is("is") {
            current_run.scorers.push(Box::new(search::scorers::Is{}));
        }
        else if arg.is("not") {
            current_run.scorers.push(Box::new(search::scorers::Not{}));
        }
        else if arg.is("has") {
            current_run.scorers.push(Box::new(search::scorers::Has{}));
        }
        else if arg.is("hasnt") {
            current_run.scorers.push(Box::new(search::scorers::Hasnt{}));
        }
        else if arg.is("more") {
            current_run.scorers.push(Box::new(search::scorers::More{}));
        }

        match arg.get_value() {
            Some(s) => { current_run.targets.push(s); },
            None => {}
        };
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
        println!("{} [insensitive={}]", run.content_loader.get_name(), run.insensitive);

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
    absolute: bool,
    score: bool,
    long: bool
}

impl FileTraverseSpecs {
    fn new(recursive: bool, hidden:bool) -> FileTraverseSpecs {
        FileTraverseSpecs{recursive: recursive,  hidden: hidden}
    }
}

impl OutputSpecs {
    fn new(absolute: bool, score: bool, long: bool) -> OutputSpecs {
        OutputSpecs{ absolute: absolute, score: score, long: long }
    }
}

fn get_file_traverse_specs(matches: &clap::ArgMatches) -> FileTraverseSpecs {
    let recursive = matches.is_present("recursive");
    let hidden = matches.is_present("hidden");

    return FileTraverseSpecs::new(recursive, hidden);
}

fn get_output_specs(matches: &clap::ArgMatches) -> OutputSpecs {
    let absolute = matches.is_present("absolute");
    let score = matches.is_present("score");
    let long = matches.is_present("long");

    return OutputSpecs::new(absolute, score, long);
}

fn get_content(run: &ContentRun, direntry: &walkdir::DirEntry) -> String {
    let mut content = run.content_loader.load_content(&direntry);
   
    if run.insensitive {
        content = content.to_ascii_lowercase();
    }

    content
}

pub fn process_command(path: &str, args: Vec<String>, matches: &clap::ArgMatches) -> u32 {
    let mut path = path::PathBuf::from(path);
    let args = parse_args(args.iter());
    //let command_order = process_command_order(args);
    let runs = get_content_runs(args.iter(), matches);

    let traverse_specs = get_file_traverse_specs(matches);
    let output_specs = get_output_specs(matches);
    let loader_names: Vec<String> = runs.iter().map(|r| String::from(r.content_loader.get_name())).collect();
    //
    //optimize_content_run_order(&mut runs);

    let mut app_stats = stats::AppStats::new();
    //let mut content_run_stats: Vec<stats::RunStats> = Vec::new();

    path = fs::canonicalize(&path).unwrap();
    if matches.is_present("echo") {
        println!("\tls {:?}", path);
        println!("\tls {:?}", path);
    }

    if matches.is_present("strats") {
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

            let content = get_content(&run, &direntry);
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
    
    print_direntries(output_specs, &path, directories);

    if matches.is_present("stats") {
        print!("{}", app_stats);
    }
    0
}

fn print_direntries(output_specs: OutputSpecs, path: &path::PathBuf, directories: Vec<(f32, walkdir::DirEntry)>) {
    if output_specs.long || output_specs.score{
        linear_print(output_specs, path, directories);
    }
    else {
        grid_print(output_specs, path, directories);
    }
}

fn path_abs(direntry: &walkdir::DirEntry) -> &str {
    direntry.path().as_os_str().to_str().unwrap()
}
fn path_rel<'a>(direntry: &'a walkdir::DirEntry, path: &'a path::PathBuf) -> &'a str {
    let dir_path = direntry.path().as_os_str().to_str().unwrap();
    match dir_path.strip_prefix(path.as_path().as_os_str().to_str().unwrap()) {
        Some(str) => if str.len() > 0 { &str[1..] } else { "" },
        None => ""
    }
}

fn print_dir<'a>(direntry: &'a walkdir::DirEntry, path: &'a path::PathBuf, absolute: bool) -> &'a str {
    if absolute {
        let dir_path = path_abs(&direntry);

        println!("{}", dir_path);
        if direntry.path().is_dir() {
            colour::green!("{}", dir_path);
        }
        else {
            colour::prnt!("{}", dir_path);
        }
        dir_path
    }
    else {
        let dir_path = path_rel(&direntry, path);
        if direntry.path().is_dir() {
            colour::green!("{}", dir_path);
        }
        else {
            colour::prnt!("{}", dir_path);
        }
        dir_path
    }
}

trait PrintlnFormatter {
    fn print(&self, score: &f32, path: &path::PathBuf, direntry: &walkdir::DirEntry, output_specs: &OutputSpecs);
}

struct ScoreFormatter { }
impl PrintlnFormatter for ScoreFormatter {
    fn print(&self, score: &f32, path: &path::PathBuf, direntry: &walkdir::DirEntry, output_specs: &OutputSpecs) {
        if output_specs.absolute {
            let dir_path = path_abs(&direntry);
            println!("[{}]{}", score, dir_path);
        }
        else {
            let clean_path = path_rel(&direntry, path); 
            println!("[{}] {}", score, clean_path);
        }
    }
}

struct LongFormatter { }
impl PrintlnFormatter for LongFormatter {
    fn print(&self, score: &f32, path: &path::PathBuf, direntry: &walkdir::DirEntry, output_specs: &OutputSpecs) {
        let meta = direntry.metadata().expect("Unable to retrieve metadata");
        let mode = meta.mode();
        let mut permission_str = String::new();

        permission_str += if meta.is_dir() { "d" } else { "-" };

        for i in 0..9 {
            let bit = mode.get_bit(i);
            if i % 3 == 0 {
                permission_str += if bit { "-" } else { "r" };
            }
            else if (i + 1) % 3 == 0 {
                permission_str += if bit { "-" } else { "x" };
            }
            else if (i + 2) % 3 == 0 {
                permission_str += if bit { "-" } else { "w" };
            }
        }

        let dir_path = if output_specs.absolute { path_abs(&direntry) } else { path_rel(&direntry, path) };
        let timestamp = meta.modified().expect("Unable to retrieve modfied").duration_since(UNIX_EPOCH).expect("Uh oh").as_secs();
        let modified = "Aug 3. 1997"; //datetime.format("%Y %m");
        let owner = meta.uid();
        let group = meta.gid();

        println!("{} {} {} {} {}", permission_str, owner, group, modified, dir_path);
    }
}

struct StdFormatter { }
impl PrintlnFormatter for StdFormatter {
    fn print(&self, score: &f32, path: &path::PathBuf, direntry: &walkdir::DirEntry, output_specs: &OutputSpecs) {
        if output_specs.absolute {
            let dir_path = path_abs(&direntry);
            println!("{}",  dir_path);
        }
        else {
            let clean_path = path_rel(&direntry, path); 
            println!("{}", clean_path);
        }
    }
}

fn linear_print(output_specs: OutputSpecs, path: &path::PathBuf, directories: Vec<(f32, walkdir::DirEntry)>) {
    let formatter: Box<dyn PrintlnFormatter> = if output_specs.score {
            Box::new(ScoreFormatter {})
        }
        else {
            //Box::new(StdFormatter {})
            Box::new(LongFormatter {})
        };

        for (score, direntry) in directories {
            formatter.print(&score, path, &direntry, &output_specs);
        }
}

fn grid_print(output_specs: OutputSpecs, path: &path::PathBuf, directories: Vec<(f32, walkdir::DirEntry)>) {
    const MAX_LINE: u32 = 80;
    if directories.len() == 0 {
        return;
    }

    let max_width: u32 = directories.iter()
        .map(|x| if output_specs.absolute { path_abs(&x.1) } else { path_rel(&x.1, path) }.len())
        .max()
        .unwrap() as u32 + 5;
    
    let columns = MAX_LINE / max_width;
    
    let mut x = 0;

    for (_score, direntry) in directories {
        if x > columns {
            x = 0;
            colour::white_ln!("");
            //println!();
        }

        if output_specs.absolute {
            let dir_path = print_dir(&direntry, path, true);
            for _x in (dir_path.len() as u32)..max_width {
                print!(" ")
            }
        }
        else {
            let dir_path = print_dir(&direntry, path, false);
            for _x in (dir_path.len() as u32)..max_width {
                print!(" ")
            }
        }

        x += 1;
    };
    std::io::stdout().flush().expect("Failed to flush stdout");
}
