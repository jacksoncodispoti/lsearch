use std::collections::HashMap;
use crate::search;
use walkdir::WalkDir;
use std::fmt;

fn get_content_loaders(loaders: std::slice::Iter<String>) -> HashMap<String, Box<dyn search::loaders::ContentLoader>> {
    let mut content_loaders: HashMap<String, Box<dyn search::loaders::ContentLoader>> = HashMap::new();

    for loader in loaders {
        //Path based
        if loader == "--content-path" {
            if !content_loaders.contains_key(&String::from(loader)) {
                content_loaders.insert(String::from(loader), Box::new(search::loaders::PathLoader::new()));
            }
        }
        else if loader == "--content-title" {
            if !content_loaders.contains_key(&String::from(loader)) {
                content_loaders.insert(String::from(loader), Box::new(search::loaders::TitleLoader::new()));
            }
        }
        else if loader == "--content-ext" {
            if !content_loaders.contains_key(&String::from(loader)) {
                content_loaders.insert(String::from(loader), Box::new(search::loaders::ExtLoader::new()));
            }
        }

        //Content-based
        else if loader == "--content-text" {
            if !content_loaders.contains_key(&String::from(loader)){
                content_loaders.insert(String::from(loader), Box::new(search::loaders::TextLoader::new()));
            }
        }
    }

    //Add default if none
    if content_loaders.len() == 0 {
        content_loaders.insert(String::from("--content-text"), Box::new(search::loaders::TextLoader::new()));
    }

    content_loaders
}

struct ContentRun {
    content_loader: String,
    scorers: Vec<Box<dyn search::scorers::ContentScorer>>,
    targets: Vec<String>
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
    let mut current_run: ContentRun = ContentRun{content_loader: String::from(current_loader), scorers: Vec::new(), targets: Vec::new()};
    let mut content_runs: Vec<ContentRun> = Vec::new();

    let mut scorer: Box<dyn search::scorers::ContentScorer> = Box::new(search::scorers::Pass{});

    for arg in args {
        if arg.starts_with("--") {
            //Content loading
            if arg.starts_with("--content") {
                current_loader = arg;

                if current_run.is_valid() {
                    content_runs.push(current_run);
                }

                current_run = ContentRun{content_loader: String::from(current_loader), scorers: Vec::new(), targets: Vec::new()};
                println!("Current loader update to {}", current_loader);
                continue;
            }

            scorer = Box::new(search::scorers::Pass{});
            //Filter/Scorer
            if arg == "--is" {
                scorer = Box::new(search::scorers::Is{});
            }
            else if arg == "--not" {
                scorer = Box::new(search::scorers::Not{});
            }
            else if arg == "--has" {
                scorer = Box::new(search::scorers::Has{});
            }
            else if arg == "--hasnt" {
                scorer = Box::new(search::scorers::Hasnt{});
            }
            else if arg == "--more" {
                scorer = Box::new(search::scorers::More{});
            }

            current_run.scorers.push(scorer);
            println!("Pushing scorer");
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
    for run in runs {
        println!("Content: {}", run.content_loader);
        
        for (scorer, target) in run.scorers.iter().zip(run.targets.iter()) {
            println!("\t{}({})", scorer.get_name(), target);
        }
    }
}

pub fn process_command(path: &str, args: Vec<String>) -> u32 {
    //let command_order = process_command_order(args);
    let runs = get_content_runs(args.iter());
    let loader_names: Vec<String> = runs.iter().map(|r| String::from(&r.content_loader)).collect();
    let content_loaders = get_content_loaders(loader_names.iter());
    //optimize_content_run_order(&mut runs);
    let mut directories: Vec<walkdir::DirEntry> = WalkDir::new(path)
        .into_iter()
        .map(|x| x.unwrap())
        .collect();

    let mut next_directories: Vec<(f32, walkdir::DirEntry)> = Vec::new();

    summarize_runs(runs.iter());

    for run in runs {
        next_directories = Vec::new();
        let dir_iter = directories.into_iter();
        for direntry in dir_iter {
            let content_loader = content_loaders.get(&run.content_loader).expect("Unable to get content loader");
            let content = content_loader.load_content(&direntry);

            let mut filtered = true;
            let mut score = 0.0;
            for (scorer, target) in run.scorers.iter().zip(run.targets.iter()) {
                let ind_score = scorer.score(&content, &target);
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
        directories = next_directories.into_iter().map(|x| x.1).collect();
    }

    for direntry in directories {
        println!("{:?}", direntry.path());
    }

    //results.sort_by(|a, b| a.partial_cmp(b).unwrap());

    //for result in results.iter().rev() {
    //    if result.0 >= 1.0 {
    //        println!("{}", result.1);
    //    }
    //    else {

    //    }
    //}

    0
}
