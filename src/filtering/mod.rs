pub mod clean;
pub mod stop_words;

const ILLEGAL_CHARS: &[char] = &[
    ',', '.', '?', '!', '-', '_', '\'', ':', '!', '"', '#', '$', '%', '&', '(', ')', '*', '+', '/',
    ';',
];

pub fn clean(s: &str) -> String {
    s.replace(ILLEGAL_CHARS, "")
}
