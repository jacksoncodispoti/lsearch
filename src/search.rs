pub mod loaders {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;
    use std::process::Command;
    use std::path;

    #[derive(Debug)]
    pub struct FileData {
        path: path::PathBuf,
    }

    impl FileData {
        pub fn new(path: path::PathBuf) -> FileData {
            FileData { path }
        }

        pub fn path(&self) -> &path::Path {
            self.path.as_path()
        }

        pub fn metadata(&self) -> std::fs::Metadata {
            self.path.metadata().expect("Unable to get metadata")
        }
    }

    pub trait ContentLoader {
        fn load_content(&self, entry: &FileData) -> String;
        fn get_name(&self) -> &str;
    }

    pub fn parse(arg: &str) -> Option<Box<dyn ContentLoader>> {
       match arg {
           "content-path" => Some(Box::new(ContentPath::new())),
           "content-text" => Some(Box::new(ContentText::new())),
           "content-title" => Some(Box::new(ContentTitle::new())),
           "content-ext" => Some(Box::new(ContentExt::new())),
           _ => None 
       }
   }

    pub struct ContentTitle { 
    }
    impl ContentTitle {
        pub fn new() -> ContentTitle {
            ContentTitle{}
        }
    }
    impl ContentLoader for ContentTitle {
        fn load_content(&self, entry: &FileData) -> String {
            String::from(match entry.path.file_name().unwrap().to_str() {
                Some(file_name) => {file_name},
                None => { "" }
            })
        }

        fn get_name(&self) -> &str {
            "content-title"
        }
    }

    pub struct ContentPath {
    }
    impl ContentPath {
        pub fn new() -> ContentPath {
            ContentPath{}
        }
    }
    impl ContentLoader for ContentPath {
        fn load_content(&self, entry: &FileData) -> String {
            String::from(entry.path.to_str().unwrap())
        }

        fn get_name(&self) -> &str {
            "content-path"
        }
    }

    pub struct ContentExt {
    }
    impl ContentExt {
        pub fn new() -> ContentExt {
            ContentExt{}
        }
    }
    impl ContentLoader for ContentExt {
        fn load_content(&self, entry: &FileData) -> String {
            match entry.path.extension() {
                Some(ext) => { return String::from(ext.to_str().unwrap());}
                None => { return String::from(""); }
            }
        }

        fn get_name(&self) -> &str {
            "content-ext"
        }
    }

    pub struct ContentText { 
    }
    impl ContentText {
        pub fn new() -> ContentText {
            ContentText{}
        }
    }
    impl ContentLoader for ContentText {
        fn load_content(&self, entry: &FileData) -> String {
            if entry.path.is_dir() {
                return String::new();
            }
            else{
                let mut contents = String::new();
                let file = File::open(String::from(entry.path.to_str().unwrap())).unwrap();
                let mut buf_reader = BufReader::new(file);
                buf_reader.read_to_string(&mut contents).expect("Failed to read contents");
                contents
            }
        }

        fn get_name(&self) -> &str {
            "content-text"
        }
    }

    pub struct ContentExec {
        command: String 
    }
    impl ContentExec {
        pub fn new(command: &'_ str) -> ContentExec {
            ContentExec{command: String::from(command)}
        }
    }
    impl ContentLoader for ContentExec {
        fn load_content(&self, entry: &FileData) -> String {
            let mut i = self.command.split(' ');
            let mut cmd = Command::new(i.next().unwrap());
            while let Some(arg) = i.next() {
                cmd.arg(arg); 
            }
            
            if let Some(file_name) = entry.path.file_name().unwrap().to_str() {
                cmd.arg(file_name);
                String::from_utf8(cmd.output().expect("Failed to run process").stdout).expect("Unable to parse output")
            }
            else {
                String::new()
            }
        }

        fn get_name(&self) -> &str {
            "content-exec"
        }
    }
}

pub mod scorers {
    pub fn create_key_from_scorer(scorer: &Box<dyn ContentScorer>, target: &String) -> String {
        create_key(&scorer.get_name(), target)
    }
    pub fn create_key(scorer: &String, target: &String) -> String {
        String::from(scorer) + "(" + target + ")"
    }

    pub mod fs {
        pub trait DirEntryFilter: std::fmt::Debug {
            fn filter(&self, content: &walkdir::DirEntry) -> bool;
        }

        #[derive(Debug)]
        pub struct HiddenFilter {
            allow: bool
        }
        impl DirEntryFilter for HiddenFilter {
            fn filter(&self, content: &walkdir::DirEntry) -> bool {
                if self.allow {
                    true
                }
                else {
                    !content.file_name().to_str().unwrap().starts_with(".")
                }
            }
        }
        impl HiddenFilter {
            pub fn _new(allow: bool) -> HiddenFilter {
                HiddenFilter{allow: allow}
            }
        }
    }

    pub trait ContentScorer: std::fmt::Debug {
        fn score(&self, content: &String, target: &String) -> f32;
        fn get_name(&self) -> String;
    }
    pub trait ContentFilter: std::fmt::Debug {
        fn filter(&self, content: &String, target: &String) -> bool;
    }

    #[derive(Debug)]
    pub struct Is {
    }
    impl ContentFilter for Is {
        fn filter(&self, content: &String, target: &String) -> bool {
            target.eq(content)
        }
    }
    impl ContentScorer for Is {
        fn score(&self, content: &String, target: &String) -> f32 {
            if self.filter(&content, &target) {1.0} else {0.0}
        }
        fn get_name(&self) -> String {
            String::from("Is")
        }
    }

    #[derive(Debug)]
    pub struct Not {
    }
    impl ContentFilter for Not {
        fn filter(&self, content: &String, target: &String) -> bool {
            !target.eq(content)
        }
    }
    impl ContentScorer for Not {
        fn score(&self, content: &String, target: &String) -> f32 {
            if self.filter(&content, &target) {1.0} else {0.0}
        }
        fn get_name(&self) -> String {
            String::from("Not")
        }
    }

    #[derive(Debug)]
    pub struct Has {
    }
    impl ContentFilter for Has {
        fn filter(&self, content: &String, target: &String) -> bool {
            for _m in content.matches(target) {
                return true
            }
            false 
        }
    }
    impl ContentScorer for Has {
        fn score(&self, content: &String, target: &String) -> f32 {
            if self.filter(&content, &target) {1.0} else {0.0}
        }
        fn get_name(&self) -> String {
            String::from("Has")
        }
    }

    #[derive(Debug)]
    pub struct Hasnt {
    }
    impl ContentFilter for Hasnt {
        fn filter(&self, content: &String, target: &String) -> bool {
            for _m in content.matches(target) {
                return false
            }
            true
        }
    }
    impl ContentScorer for Hasnt {
        fn score(&self, content: &String, target: &String) -> f32 {
            if self.filter(&content, &target) {1.0} else {0.0}
        }
        fn get_name(&self) -> String {
            String::from("Hasnt")
        }
    }

    #[derive(Debug)]
    pub struct More {
    }
    impl ContentScorer for More {
        fn score(&self, content: &String, target: &String) -> f32 {
            let mut score = 1.0;

            for _m in content.matches(target) {
                score += 1.0;
            }

            score
        }
        fn get_name(&self) -> String {
            String::from("More")
        }
    }

    #[derive(Debug)]
    pub struct Pass {

    }
    impl ContentScorer for Pass {
        fn score(&self, _content: &String, _target: &String) -> f32 {
            1.0
        }
        fn get_name(&self) -> String {
            String::from("Pass")
        }
    }
}
