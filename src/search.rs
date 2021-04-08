pub mod loaders {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;
    pub trait ContentLoader {
        fn load_content(&self, entry: &walkdir::DirEntry) -> String;
    }

    pub struct TitleLoader { 
    }
    impl TitleLoader {
        pub fn new() -> TitleLoader {
            TitleLoader{}
        }
    }
    impl ContentLoader for TitleLoader {
        fn load_content(&self, entry: &walkdir::DirEntry) -> String {
            String::from(match entry.file_name().to_str() {
                Some(file_name) => {file_name},
                None => { "" }
            })
        }
    }

    pub struct PathLoader {
    }
    impl PathLoader {
        pub fn new() -> PathLoader {
            PathLoader{}
        }
    }
    impl ContentLoader for PathLoader {
        fn load_content(&self, entry: &walkdir::DirEntry) -> String {
            String::from(entry.path().to_str().unwrap())
        }
    }

    pub struct ExtLoader {
    }
    impl ExtLoader {
        pub fn new() -> ExtLoader {
            ExtLoader{}
        }
    }
    impl ContentLoader for ExtLoader {
        fn load_content(&self, entry: &walkdir::DirEntry) -> String {
            match entry.path().extension() {
                Some(ext) => { return String::from(ext.to_str().unwrap());}
                None => { return String::from(""); }
            }
        }
    }

    pub struct TextLoader { 
    }
    impl TextLoader {
        pub fn new() -> TextLoader {
            TextLoader{}
        }
    }
    impl ContentLoader for TextLoader {
        fn load_content(&self, entry: &walkdir::DirEntry) -> String {
            if entry.path().is_dir() {
                return String::new();
            }
            else{
                println!("{}", entry.path().to_str().unwrap());
                let mut contents = String::new();
                let file = File::open(String::from(entry.path().to_str().unwrap())).unwrap();
                let mut buf_reader = BufReader::new(file);
                buf_reader.read_to_string(&mut contents);
                contents
            }
        }
    }
}

pub mod scorers {
    pub trait ContentScorer {
        fn score(&self, content: &String, target: &String) -> f32;
        fn get_name(&self) -> String;
    }
    pub trait ContentFilter {
        fn filter(&self, content: &String, target: &String) -> bool;
    }
    pub struct SuperContentScorer<'a> {
        pub scorer: Box<dyn ContentScorer>,
        pub content: &'a str,
    }
    impl SuperContentScorer<'_> {
        pub fn new<'a>(scorer: Box<dyn ContentScorer>, content: &'a str) -> SuperContentScorer<'a> {
            SuperContentScorer{scorer: scorer, content: content}
        }
    }

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
