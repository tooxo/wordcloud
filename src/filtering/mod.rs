mod stop_word_iterators;
mod stop_words;

pub use stop_word_iterators::{StopWordsIteratorPar, StopWordsIterator};
pub use stop_words::StopWords;

const ILLEGAL_CHARS: &[char] = &[
    ',', '.', '?', '!', '-', '_', '\'', ':', '!', '"', '#', '$', '%', '&', '(', ')', '*', '+', '/',
    ';',
];

/**
    Remove symbols from a given string
*/
pub fn clean(s: &str) -> String {
    s.replace(ILLEGAL_CHARS, "")
}
