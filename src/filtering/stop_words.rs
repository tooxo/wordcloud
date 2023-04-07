use crate::io::file::read_string_from_file;
use std::collections::{HashMap, HashSet};
use unicode_script::{Script, UnicodeScript};

pub struct StopWords {
    stop_word_map: HashMap<Script, HashSet<String>>,
}

impl StopWords {
    fn used_scripts(word: &str) -> HashSet<Script> {
        word.chars()
            .map(|c| c.script())
            .collect::<HashSet<Script>>()
    }

    pub fn is_included(&self, word: &str) -> bool {
        let scripts = StopWords::used_scripts(word);
        let l: String = word.trim().to_lowercase();
        for unicode_script in &scripts {
            if let Some(mi) = self.stop_word_map.get(unicode_script) {
                if mi.contains(&l) {
                    return true;
                }
            }
        }

        false
    }

    pub fn new() -> Self {
        Self {
            stop_word_map: Default::default(),
        }
    }

    pub fn append_words<'a>(&mut self, words: impl Iterator<Item = &'a str>) {
        let filtered_words = words
            .map(|x| x.trim().to_lowercase())
            .filter(|x| !x.is_empty())
            .map(|x| (StopWords::used_scripts(&x), x));

        for (scripts, word) in filtered_words {
            for sc in &scripts {
                if !self.stop_word_map.contains_key(sc) {
                    self.stop_word_map.insert(*sc, HashSet::new());
                }
                self.stop_word_map.get_mut(sc).unwrap().insert(word.clone());
            }
        }
    }

    pub fn add_words<'a>(mut self, words: impl Iterator<Item = &'a str>) -> Self {
        self.append_words(words);
        self
    }

    pub fn append_file(&mut self, filename: &str) {
        let contents = read_string_from_file(filename);

        let iter = contents.split_whitespace();

        self.append_words(iter)
    }

    pub fn add_file(mut self, filename: &str) -> Self {
        self.append_file(filename);
        self
    }
}
