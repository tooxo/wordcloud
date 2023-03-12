use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn read_string_from_file(filename: &str) -> String {
    let path = Path::new(filename);
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => panic!("file not found"),
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(size) => size,
        Err(_e) => panic!("couldn't read file"),
    };

    contents
}
