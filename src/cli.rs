use std::collections::HashMap;
use crate::search;

fn get_content_loaders(args: std::slice::Iter<String>) -> HashMap<String, Box<dyn search::loaders::ContentLoader>> {
    let mut content_loaders: HashMap<String, Box<dyn search::loaders::ContentLoader>> = HashMap::new();

    for arg in args {
        if arg == "--content-text" {
            if !content_loaders.contains_key(&String::from(arg)){
                content_loaders.insert(String::from(arg), Box::new(search::loaders::TextLoader::new()));
            }
        }

        if arg == "--content-title" {
            if !content_loaders.contains_key(&String::from(arg)) {
                content_loaders.insert(String::from(arg), Box::new(search::loaders::TitleLoader::new()));
            }
        }

        if arg == "--content-path" {
            if !content_loaders.contains_key(&String::from(arg)) {
                content_loaders.insert(String::from(arg), Box::new(search::loaders::PathLoader::new()));
            }
        }

        if arg == "--content-ext" {
            if !content_loaders.contains_key(&String::from(arg)) {
                content_loaders.insert(String::from(arg), Box::new(search::loaders::ExtLoader::new()));
            }
        }
    }

    if content_loaders.len() == 0 {
        content_loaders.insert(String::from("--content-text"), Box::new(search::loaders::TitleLoader::new()));
    }

    content_loaders
}

//fn process_command_order(args: Vec<String>) -> Vec<SuperContentScorer> {
//    let mut commands: Vec::<SuperContentScorer> = Vec::new();

//    let mut iter = args.iter().peekable();
//    let content_loaders = get_content_loaders(args.iter());
//    let mut current_loader = "--content-text";

//    while let Some(arg) = iter.next() {
//        if arg == "--content-text" {
//            current_loader = arg;
//        }
//        if arg == "--content-title" {
//                current_loader = arg;
//        } 
//        if arg == "--more" {
//            match iter.peek() {
//                Some(&next) => {
//                    let b = Box::new(search::More{target: next.clone()});
//                    let content = String::from(content_loaders.get(&String::from(current_loader)).unwrap().load_content());
//                    commands.push(SuperContentScorer::new(b, content));
//                },
//                None => { panic!("Expected a term after more"); }
//            }
//        }
//        else if arg == "--is" {
//            match iter.peek() {
//                Some(&next) => {
//                    let b = Box::new(search::Is{target: next.clone() });
//                    let content = content_loaders.get(&String::from(current_loader)).unwrap().load_content();
//                    commands.push(SuperContentScorer::new(b, &content));
//                },
//                None => { panic!("Expected a term after more"); }
//            }
//        }
//    }

//    commands
//}

use walkdir::WalkDir;
pub fn process_command(args: Vec<String>) -> u32 {
    //let command_order = process_command_order(args);
    let content_loaders = get_content_loaders(args.iter());
    let mut current_loader = "--content-text";

    let mut results: Vec<(f32, String)> = Vec::new();


    for direntry in WalkDir::new(".") {
        let direntry = direntry.unwrap();
        let mut commands: Vec::<search::scorers::SuperContentScorer> = Vec::new();
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            //println!("MASTER {}", arg);
            //while let Some(arg) = iter.next() {
            if arg.starts_with("--content") {
                current_loader = arg;
                continue;
            }

            if arg == "--more" {
                match iter.next() {
                    Some(next) => {
                        let content_loader = content_loaders.get(&String::from(current_loader)).expect("Content not loaded");
                        let content = content_loader.load_content(&direntry);
                        let b = Box::new(search::scorers::More{target: next.clone()});
                        commands.push(search::scorers::SuperContentScorer::new(b, String::from(content)));
                    },
                    None => { panic!("Expected a term after more"); }
                }
            }
            else if arg == "--is" {
                match iter.next() {
                    Some(next) => {
                        let content_loader = content_loaders.get(&String::from(current_loader)).expect("Content not loaded");
                        let content = content_loader.load_content(&direntry);
                        let b = Box::new(search::scorers::Is{target: next.clone() });
                        commands.push(search::scorers::SuperContentScorer::new(b, String::from(content)));
                    },
                    None => { panic!("Expected a term after more"); }
                }
            }
            else if arg == "--has" {
                match iter.next() {
                    Some(next) => {
                        let content_loader = content_loaders.get(&String::from(current_loader)).expect("Content not loaded");
                        let content = content_loader.load_content(&direntry);
                        let b = Box::new(search::scorers::Has{target: next.clone() });
                        commands.push(search::scorers::SuperContentScorer::new(b, String::from(content)));
                    },
                    None => { panic!("Expected a term after more"); }
                }
            }
        }

        let mut score = 0.0;
        let mut okay = true;

        for cmd in commands {
            let content = String::from(cmd.content);
            let local_score = cmd.scorer.score(&content);
            score += local_score;

            if local_score < 1.0 {
                okay = false;
                break;
            }
        }

        if okay {
            results.push((score, String::from(direntry.path().as_os_str().to_str().unwrap())));
        }
    }

    results.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for result in results.iter().rev() {
        if result.0 >= 1.0 {
            println!("{}", result.1);
        }
        else {

        }
    }

    0
}
