pub mod loaders {
    use std::fs;
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
            String::from(entry.file_name().to_str().unwrap())
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
                fs::read_to_string(entry.path()).unwrap()
            }
        }
    }
}

pub mod scorers {
    pub trait ContentScorer {
        fn score(&self, content: &String) -> f32;
    }
    pub trait ContentFilter {
        fn filter(&self, content: &String) -> bool;
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

    pub struct Is {
        pub target: String
    }
    impl ContentFilter for Is {
        fn filter(&self, content: &String) -> bool {
            self.target.eq(content)
        }
    }
    impl ContentScorer for Is {
        fn score(&self, content: &String) -> f32 {
            if self.filter(&content) {1.0} else {0.0}
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
    impl ContentScorer for Has {
        fn score(&self, content: &String) -> f32 {
            if self.filter(&content) {1.0} else {0.0}
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
}
