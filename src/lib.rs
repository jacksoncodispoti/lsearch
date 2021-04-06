pub mod cli {
    use crate::search::{self, SuperContentScorer, ContentLoader};
    use std::collections::HashMap;

    fn get_default_content_loader() -> Box<dyn ContentLoader> {
        Box::new(search::TextLoader::new())
    }

    fn get_content_loaders(args: std::slice::Iter<String>) -> HashMap<String, Box<dyn ContentLoader>> {
        let mut content_loaders: HashMap<String, Box<dyn ContentLoader>> = HashMap::new();

        for arg in args {
            if arg == "--content-text" {
                if !content_loaders.contains_key(&String::from(arg)){
                    content_loaders.insert(String::from(arg), Box::new(search::TextLoader::new()));
                }
            }

            if arg == "--content-title" {
                if !content_loaders.contains_key(&String::from(arg)) {
                    content_loaders.insert(String::from(arg), Box::new(search::TitleLoader::new()));
                }
            }

            if arg == "--content-path" {
                if !content_loaders.contains_key(&String::from(arg)) {
                    content_loaders.insert(String::from(arg), Box::new(search::PathLoader::new()));
                }
            }
        }

        if content_loaders.len() == 0 {
                content_loaders.insert(String::from("--content-text"), Box::new(search::TitleLoader::new()));
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
        let mut iter = args.iter().peekable();
        let content_loaders = get_content_loaders(args.iter());
        let mut current_loader = "--content-text";
        
        let mut results: Vec<(f32, String)> = Vec::new();


        for direntry in WalkDir::new(".") {
            let mut scorers = 0.0;
            let direntry = direntry.unwrap();
            let mut commands: Vec::<SuperContentScorer> = Vec::new();
            let mut iter = args.iter();

            while let Some(arg) = iter.next() {
                //println!("MASTER {}", arg);
                //while let Some(arg) = iter.next() {
                if arg.starts_with("--content") {
                    if arg == "--content-text" {
                        current_loader = arg;
                    }
                    if arg == "--content-title" {
                        current_loader = arg;
                    } 
                    if arg == "--content-path" {
                        current_loader = arg;
                    }
                    continue;
                }

                if arg == "--more" {
                    match iter.next() {
                        Some(next) => {
                            let content_loader = content_loaders.get(&String::from(current_loader)).expect("Content not loaded");
                            let content = content_loader.load_content(&direntry);
                            let b = Box::new(search::More{target: next.clone()});
                            commands.push(SuperContentScorer::new(b, String::from(content)));
                        },
                        None => { panic!("Expected a term after more"); }
                    }
                    scorers += 1.0;
                }
                else if arg == "--is" {
                    match iter.next() {
                        Some(next) => {
                            let content_loader = content_loaders.get(&String::from(current_loader)).expect("Content not loaded");
                            let content = content_loader.load_content(&direntry);
                            let b = Box::new(search::Is{target: next.clone() });
                            commands.push(SuperContentScorer::new(b, String::from(content)));
                        },
                        None => { panic!("Expected a term after more"); }
                    }
                    scorers += 1.0;
                }
                else if arg == "--has" {
                    match iter.next() {
                        Some(next) => {
                            let content_loader = content_loaders.get(&String::from(current_loader)).expect("Content not loaded");
                            let content = content_loader.load_content(&direntry);
                            let b = Box::new(search::Has{target: next.clone() });
                            commands.push(SuperContentScorer::new(b, String::from(content)));
                        },
                        None => { panic!("Expected a term after more"); }
                    }
                    scorers += 1.0;
                }
            }

            let mut score = 0.0;
            let mut okay = true;

            for cmd in commands {
                let content = String::from(cmd.content);
                let local_score = cmd.scorer.score(&content);
                score += local_score;

                if local_score == 0.0 {
                    okay = false;
                    break;
                }
            }

            if okay {
                results.push((score, String::from(direntry.path().as_os_str().to_str().unwrap())));
            }
            ////Filtered out
            //if score / 1.0 * scorers < 1.0 {

            //}
            //else {
            //    results.push((score, String::from(direntry.path().as_os_str().to_str().unwrap())));
            //}

        }

        results.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for result in results.iter().rev() {
            if result.0 >= 1.0 {
                println!("{}", result.1)
            }
            else {

            }
            //println!("{} {}", result.1, result.0);
        }

        0
    }
}

pub mod search {
    use std::fs;
    use std::path::Path;
    use std::cmp;
    use std::os::unix::fs::MetadataExt;

    pub trait MetadataScorer {
        fn score(&self, meta_data: &fs::Metadata) -> f32;
    }

    use walkdir;
    pub trait ContentLoader {
        fn load_content(&self, entry: &walkdir::DirEntry) -> String;
    }

    #[derive(Debug)]
    pub struct TitleLoader { 
        pub content: String,
    }

    impl TitleLoader {
        pub fn new() -> TitleLoader {
            TitleLoader{content: String::new()}
        }
    }

    impl ContentLoader for TitleLoader {
        fn load_content(&self, entry: &walkdir::DirEntry) -> String {
            String::from(entry.file_name().to_str().unwrap())
        }
    }

    pub struct PathLoader {
        pub content: String,
    }

    impl PathLoader {
        pub fn new() -> PathLoader {
            PathLoader{content: String::new()}
        }
    }

    impl ContentLoader for PathLoader {
        fn load_content(&self, entry: &walkdir::DirEntry) -> String {
            String::from(entry.path().to_str().unwrap())
        }
    }

    #[derive(Debug)]
    pub struct TextLoader { 
        content: String
    }

    impl TextLoader {
        pub fn new() -> TextLoader {
            TextLoader{content: String::new()}
        }
        fn load_content(&self, entry: &walkdir::DirEntry) {
        }
    }
    impl ContentLoader for TextLoader {
        fn load_content(&self, entry: &walkdir::DirEntry) -> String {
            if entry.path().is_dir() {
                return String::new();
            }
            else{
                fs::read_to_string(entry.path()).unwrap()
            }
        }
    }

    pub struct SuperContentScorer {
        pub scorer: Box<dyn ContentScorer>,
        pub content: String,
    }

    impl SuperContentScorer {
        pub fn new(scorer: Box<dyn ContentScorer>, content: String) -> SuperContentScorer {
            SuperContentScorer{scorer: scorer, content: content}
        }
    }

    pub trait ContentScorer {
        fn score(&self, content: &String) -> f32;
    }

    pub trait ContentFilter {
        fn filter(&self, content: &String) -> bool;
    }

    impl ContentScorer for Is {
        fn score(&self, content: &String) -> f32{
            if self.filter(content) {
                return 1.0
            }
            else {
                return -1.0
            }
        }
    }

    impl ContentScorer for Has {
        fn score(&self, content: &String) -> f32{
            if self.filter(content) {
                return 1.0
            }
            else {
                return -1.0
            }
        }

    }

    pub struct More {
        pub target: String
    }

    impl ContentScorer for More {
        fn score(&self, content: &String) -> f32 {
            let mut score = 0.0;

            for _m in content.matches(&self.target) {
                score += 1.0;
            }

            score
        }
    }

    pub struct Is {
        pub target: String
    }

    impl ContentFilter for Is {
        fn filter(&self, content: &String) -> bool {
            //println!("Compareing [{}]==[{}]", self.target, content);
            self.target.eq(content)
        }
    }

    pub struct Has {
        pub target: String
    }

    impl ContentFilter for Has {
        fn filter(&self, content: &String) -> bool {
            for _m in content.matches(&self.target) {
                return true
            }
            false 
        }
    }

    impl ContentScorer for dyn ContentFilter {
        fn score(&self, content:&String) -> f32 {
            if self.filter(&content) {
                return 1.0
            }
            else{
                return 0.0
            }
        }
    }

    pub struct ScoredDirEntry {
        pub entry: fs::DirEntry,
        pub score: f32,
    }

    impl Ord for ScoredDirEntry {
        fn cmp(&self, other: &Self) -> cmp::Ordering {
            match self.partial_cmp(&other) {
                Some(ord) => ord,
                None => cmp::Ordering::Equal
            }
        }
    }

    impl PartialOrd for ScoredDirEntry {
        fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
            self.score.partial_cmp(&other.score)
        }
    }

    impl PartialEq for ScoredDirEntry {
        fn eq(&self, other: &Self) -> bool {
            self.score == other.score
        }
    }

    impl Eq for ScoredDirEntry {}

    pub struct PermissionScoreSpec {
        pub permission: u32,
    }

    pub struct DirScoreSpec {
        pub is_dir: bool,
    }

    impl MetadataScorer for PermissionScoreSpec {
        fn score(&self, metadata: &fs::Metadata) -> f32 {
            println!("{}",metadata.mode());
            if metadata.mode() == self.permission{
                1.0
            }
            else {
                0.0
            }
        }
    }

    impl MetadataScorer for DirScoreSpec {
        fn score(&self, metadata: &fs::Metadata) -> f32 {
            if metadata.is_dir() == self.is_dir {
                1.0
            }
            else {
                0.0
            }
        }
    }

    pub fn metadata_search(root_dir: &Path, spec: &impl MetadataScorer) -> Vec<ScoredDirEntry> {
        let mut scored_files = metadata_score_files(&root_dir, spec);
        scored_files.sort();
        scored_files.reverse();
        scored_files
    }

    fn metadata_score_files(root_dir: &Path, spec: &impl MetadataScorer) -> Vec<ScoredDirEntry> {
        let mut vec: Vec<ScoredDirEntry> = vec![];

        if root_dir.is_dir() {
            for entry in fs::read_dir(root_dir).unwrap() {
                let entry = match entry {
                    Ok(file) => file,
                    Err(error) => panic!("Problem reading the file {:?}", error),
                };

                let metadata = entry.metadata().unwrap();
                let score = spec.score(&metadata);

                vec.push(ScoredDirEntry{entry: entry, score: score});
            }
        }

        vec
    }
}
