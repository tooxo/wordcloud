use std::fs::{canonicalize, read_dir};
use std::iter::FusedIterator;
use std::path::{Path, PathBuf};

pub struct RecursiveFolderIterator<'a> {
    folder: Vec<PathBuf>,
    file: Vec<PathBuf>,
    filter_fn: &'a dyn Fn(&PathBuf) -> bool,
}

impl<'a> RecursiveFolderIterator<'a> {
    pub fn new(folder: &Path, filter: &'a dyn Fn(&PathBuf) -> bool) -> Self {
        RecursiveFolderIterator {
            folder: vec![folder.canonicalize().unwrap()],
            file: Vec::new(),
            filter_fn: filter,
        }
    }
}

impl<'a> Iterator for RecursiveFolderIterator<'a> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            while let Some(file) = self.file.pop() {
                if (self.filter_fn)(&file) {
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

impl<'a> FusedIterator for RecursiveFolderIterator<'a> {}
