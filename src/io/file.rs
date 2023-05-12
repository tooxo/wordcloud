use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

/**
    Reads the contents of a file to a string.
*/
pub fn read_string_from_file(filename: &str) -> io::Result<String> {
    let path = Path::new(filename);
    let mut file = File::open(path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}
