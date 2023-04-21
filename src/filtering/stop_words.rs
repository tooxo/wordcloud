use crate::io::file::read_string_from_file;
use std::collections::{HashMap, HashSet};
use unicode_script::{Script, UnicodeScript};

pub struct StopWords {
    stop_word_map: HashMap<Script, HashSet<String>>,
}

impl StopWords {
    fn used_script(word: &str) -> Script {
        match word.chars().next() {
            None => Script::Unknown,
            Some(c) => c.script(),
        }
    }

    pub fn is_included(&self, word: &str) -> bool {
        let script = StopWords::used_script(word);
        let l: String = word.trim().to_lowercase();
        let words = self.stop_word_map.get(&script);
        match words {
            None => false,
            Some(w) => w.contains(&l),
        }
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
            .map(|x| (StopWords::used_script(&x), x));

        for (script, word) in filtered_words {
            let entry = self
                .stop_word_map
                .entry(script)
                .or_insert_with(HashSet::new);
            entry.insert(word.clone());
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
