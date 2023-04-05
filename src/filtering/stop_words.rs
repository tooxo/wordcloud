use crate::io::file::read_string_from_file;
use std::collections::HashSet;

pub struct StopWords {
    stop_word_list: HashSet<String>,
}

impl StopWords {
    pub fn is_included(&self, word: &str) -> bool {
        let l: String = word.trim().to_lowercase();
        self.stop_word_list.contains(&l)
    }

    fn new(stop_word_list: Vec<String>) -> Self {
        Self {
            stop_word_list: HashSet::from_iter(stop_word_list.into_iter()),
        }
    }

    pub fn from_file(filename: &str) -> StopWords {
        let contents = read_string_from_file(filename);

        let iter = contents.split('\n');

        let mut v: Vec<String> = Vec::new();
        for x in iter {
            v.push(String::from(x.to_lowercase().trim()));
        }

        StopWords::new(v)
    }
}
