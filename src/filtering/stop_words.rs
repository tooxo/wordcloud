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
        scripts
            .iter()
            .filter_map(|s| self.stop_word_map.get(s))
            .map(|mi| mi.contains(&l))
            .any(|x| x)
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

    pub fn append_file(&mut self, filename: &str) {
        let contents = read_string_from_file(filename);
        let iter = contents.split_whitespace();

        self.append_words(iter)
    }
}

#[cfg(feature = "stopwords")]
use include_dir::{include_dir, Dir};

#[cfg(feature = "stopwords")]
impl Default for StopWords {
    fn default() -> Self {
        static DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/assets/stopwords-json");

        let mut sw = StopWords::new();

        for x in DIR.find("*.txt").unwrap() {
            sw.append_words(
                x.as_file()
                    .unwrap()
                    .contents_utf8()
                    .unwrap()
                    .split_whitespace(),
            );
        }

        sw
    }
}
