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

pub struct ExtLoader {
    pub content: String,
}

impl ExtLoader {
    pub fn new() -> ExtLoader {
        ExtLoader{content: String::new()}
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

#[derive(Debug)]
pub struct TextLoader { 
    content: String
}

impl TextLoader {
    pub fn new() -> TextLoader {
        TextLoader{content: String::new()}
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
