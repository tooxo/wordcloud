pub mod stop_words;
pub mod clean;

const ILLEGAL_CHARS: &[char] = &[
    ',', '.', '?', '!', '-', '_', '\'', ':', '!', '"', '#', '$', '%', '&', '(', ')', '*', '+', '/', ';'];

pub fn clean(s: &str) -> String {
    s.replace(ILLEGAL_CHARS, "")
}