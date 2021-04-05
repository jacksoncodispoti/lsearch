pub mod search {
    use std::fs;
    use std::path::Path;
    use std::cmp;
    use std::os::unix::fs::MetadataExt;

    pub trait MetadataScorer {
        fn score(&self, meta_data: &fs::Metadata) -> f32;
    }

    pub trait ContentScorer {
        fn score(&self, content: &String) -> f32;
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
