use crate::io::read_string_from_file;
use std::collections::{HashMap, HashSet};
use std::io;

use unicode_script::{Script, UnicodeScript};

/**
    Keeps track of all used StopWords. Supports categorizing by writing scripts.
*/
#[derive(Debug)]
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

    /**
        Checks, if a given `word` is included in the StopWords database.
    */
    pub fn is_included(&self, word: &str) -> bool {
        let script = StopWords::used_script(word);
        let l: String = word.trim().to_lowercase();
        let words = self.stop_word_map.get(&script);
        match words {
            None => false,
            Some(w) => w.contains(&l),
        }
    }

    /**
        Create an empty StopWord database.
    */
    pub fn new() -> Self {
        Self {
            stop_word_map: Default::default(),
        }
    }

    /**
        Appends an iterator of `words` to the database.
    */
    pub fn append_words_iter<'a>(&mut self, words_iter: impl Iterator<Item = &'a str>) {
        let filtered_words = words_iter
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

    /**
        Appends a collection of `words` to the database.
    */
    pub fn append_words<'a>(&mut self, words: impl IntoIterator<Item = &'a str>) {
        self.append_words_iter(words.into_iter())
    }

    /**
        Appends the content of a file to the database.
    */
    pub fn append_file(&mut self, filename: &str) -> io::Result<()> {
        let contents = read_string_from_file(filename)?;
        let iter = contents.split_whitespace();

        self.append_words(iter);
        Ok(())
    }
}

#[cfg(feature = "stopwords")]
use include_dir::{include_dir, Dir};

#[cfg(feature = "stopwords")]
impl Default for StopWords {
    /**
        If the feature ["stopwords"] is provided, a default list of StopWords from the
        stopwords-json repository can be included.
    */
    fn default() -> Self {
        static DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/assets/stopwords-json");

        let mut sw = StopWords::new();

        for x in DIR.find("*.txt").expect("stopwords: iteration error") {
            sw.append_words(
                x.as_file()
                    .expect("couldn't parse as file")
                    .contents_utf8()
                    .unwrap_or("")
                    .split_whitespace(),
            );
        }

        sw
    }
}
