#![feature(iter_intersperse)]
#![feature(box_into_inner)]

pub mod filtering;
pub mod io;
pub mod rank;
pub mod cloud;
mod common;
mod image;
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
