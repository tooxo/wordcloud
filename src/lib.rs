#![feature(iter_intersperse)]
#![feature(box_into_inner)]

pub mod cloud;
mod common;
pub mod filtering;
mod image;
pub mod io;
pub mod rank;
mod types;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
