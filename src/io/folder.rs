use std::fs::{canonicalize, read_dir};
use std::iter::FusedIterator;
use std::path::{Path, PathBuf};

/**
Recursively iterates through items in the given folder, which match the given filter
 */
pub struct RecursiveFolderIterator {
    folder: Vec<PathBuf>,
    file: Vec<PathBuf>,
    filter_fn: fn(&Path) -> bool,
}

impl RecursiveFolderIterator {
    pub fn new(folder: &Path, filter: fn(&Path) -> bool) -> Self {
        RecursiveFolderIterator {
            folder: vec![folder.canonicalize().unwrap()],
            file: Vec::new(),
            filter_fn: filter,
        }
    }
}

impl Iterator for RecursiveFolderIterator {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            while let Some(file) = self.file.pop() {
                if (self.filter_fn)(file.as_path()) {
                    return Some(file);
                }
            }

            if let Some(folder) = self.folder.pop() {
                if let Ok(fl) = read_dir(folder) {
                    for f in fl.flatten() {
                        let pth = canonicalize(f.path()).unwrap();
                        if pth.is_dir() {
                            self.folder.push(pth);
                        } else if pth.is_file() {
                            self.file.push(pth);
                        }
                    }
                };
            } else {
                break;
            }
        }

        None
    }
}

impl FusedIterator for RecursiveFolderIterator {}
