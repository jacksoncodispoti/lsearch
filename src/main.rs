use std::fs;
use std::io;
use std::io::Write;
use std::path;
use std::cmp::Ordering;
use clap::{Arg, App};

struct Count {
    word: String,
    count: usize
}

impl PartialOrd for Count {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Count {
    fn cmp(&self, other: &Self) -> Ordering {
        self.count.cmp(&other.count)
    }
}

impl PartialEq for Count {
    fn eq(&self, other: &Self) -> bool {
        self.word == other.word && self.count == other.count
    }
}

impl Eq for Count {
}

struct Index {
    dir: path::PathBuf,
    counts: Vec<Count>,
}

impl Index {
    fn score(&self, keyword: &String) -> Option<usize> {
        for count in &self.counts {
            if keyword == &count.word {
                return Some(count.count)
            }
        }

        None
    }

    fn print(&self) {
        for count in &self.counts {
            println!("{} {}", count.word, count.count);
        }
    }
}

fn index(path: &fs::DirEntry, sensitive: bool) -> Index {
    let mut contents = fs::read_to_string(path.path())
        .expect("Failed to read file");

    if !sensitive {
        contents = contents.to_ascii_lowercase();
    }

    contents = contents.replace(",", " ");
    let mut words = contents.split_whitespace()
        .collect::<Vec<&str>>();

    words.sort();

    let mut counts: Vec<Count> = vec![];
    for &word in &words {
        match counts.last_mut() {
            Some(count) => {
                if &count.word == word {
                    count.count +=1;
                }
                else{
                    counts.push(Count{word: String::from(word), count: 1})
                }
            },
            None => counts.push(Count{word: String::from(word), count: 1})
        }
    }

    counts.sort();
    Index{dir: path.path(), counts: counts}
}

fn search(path: &fs::DirEntry, keyword: &String) -> usize {
    let contents = fs::read_to_string(path.path())
        .expect("Could not open file");

    contents.matches(keyword).count()
}

fn main() {
    let matches = App::new("L-Search")
        .version("0.0.1")
        .author("Alerik <alerik@alerik.de>")
        .about("Search through ALL files");

    let mut indices: Vec<Index> = vec![];
    let mut file_count = 0;

    for path in fs::read_dir(".").unwrap() {
        indices.push(index(&path.unwrap(), false));
        file_count += 1;
    }

    println!("Indexed {} files", file_count);

    loop {
        print!("Please enter a keyword to search. $ ");
        io::stdout().flush().unwrap();

        let mut keyword = String::new();
        io::stdin()
            .read_line(&mut keyword)
            .expect("Failed to readline");

        keyword = keyword.trim().to_string();

        let mut file_counts: Vec<Count> = vec![];
        for index in &indices {
            let path = index.dir.as_path();
            let path = String::from(path.as_os_str().to_str().unwrap());
            match index.score(&keyword) {
                Some(score) => file_counts.push(Count{word:path, count:score}),
                None => {},
            }
        }

        file_counts.sort();

        println!("Results for search [{}]", keyword);

        if file_counts.len() == 0 {
            println!("No results!");
        }

        for count in file_counts.iter().rev() {
            println!("\t{}: {}", count.word, count.count);
        }

        println!();
    }
}
