pub mod clean;
mod stop_words;

pub use stop_words::StopWords;

const ILLEGAL_CHARS: &[char] = &[
    ',', '.', '?', '!', '-', '_', '\'', ':', '!', '"', '#', '$', '%', '&', '(', ')', '*', '+', '/',
    ';',
];

pub fn clean(s: &str) -> String {
    s.replace(ILLEGAL_CHARS, "")
}
