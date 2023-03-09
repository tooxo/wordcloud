use std::ops::{Add, Sub};
use crate::common::svg_command::Line;
use crate::types::point::Point;

#[derive(Copy, Clone, Debug)]
pub(crate) struct Rect<T> {
    pub(crate) min: Point<T>,
    pub(crate) max: Point<T>,
}

impl<T> Add<Rect<T>> for Rect<T> where T: Add<Output=T> {
    type Output = Self;

    fn add(self, rhs: Rect<T>) -> Self::Output {
        Rect {
            min: self.min + rhs.min,
            max: self.max + rhs.max,
        }
    }
}

impl<T> Add<Point<T>> for Rect<T> where T: Add<Output=T> + Copy {
    type Output = Rect<T>;

    fn add(self, rhs: Point<T>) -> Self::Output {
        Rect {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl<T> Sub<Rect<T>> for Rect<T> where T: Sub<Output=T> {
    type Output = Self;

    fn sub(self, rhs: Rect<T>) -> Self::Output {
        Rect {
            min: self.min - rhs.min,
            max: self.max - rhs.max,
        }
    }
}

impl<T> Default for Rect<T> where T: Default {
    fn default() -> Self {
        Rect {
            min: Point::<T>::default(),
            max: Point::<T>::default(),
        }
    }
}

impl<T> Rect<T> where T: PartialOrd {
    pub(crate) fn overlaps(&self, other: &Rect<T>) -> bool {
        !(self.max.x < other.min.x || self.max.y < other.min.y || self.min.x > other.max.x || self.min.y > other.max.y)
    }

    pub(crate) fn contains(&self, other: &Rect<T>) -> bool {
        self.min.x <= other.min.x
            && self.min.y <= other.min.y
            && self.max.x >= other.max.x
            && self.max.y >= other.max.y
    }
    pub(crate) fn intersects(&self, other: &Line<T>) -> bool {
        if (other.start.x <= self.min.x && other.end.x <= self.min.x) ||
            (other.start.y <= self.min.y && other.end.y <= self.min.y) ||
            (other.start.x >= self.max.x && other.end.x >= self.max.x) ||
            (other.start.y >= self.max.y && other.end.y >= self.max.y) {
            return false;
        }
        true
    }

    pub(crate) fn is_normal(&self) -> bool {
        self.min.full_le(&self.max)
    }

    pub(crate) fn normalize(&mut self) where T: PartialOrd + Copy {
        let min_x = if self.min.x <= self.max.x { self.min.x } else { self.max.x };
        let min_y = if self.min.y <= self.max.y { self.min.y } else { self.max.y };
        let max_x = if self.min.x >= self.max.x { self.min.x } else { self.max.x };
        let max_y = if self.min.y >= self.max.y { self.min.y } else { self.max.y };

        self.min = Point { x: min_x, y: min_y };
        self.max = Point { x: max_x, y: max_y };
    }

    pub(crate) fn extend(&self, thickness: T) -> Self where T: Copy + PartialOrd + Sub<Output=T> + Add<Output=T> {
        let mut extended = *self;
        if self.is_normal() {
            extended.normalize();
        }
        extended.min = extended.min - Point { x: thickness, y: thickness };
        extended.max = extended.max + Point { x: thickness, y: thickness };

        extended
    }
}

impl<T> Rect<T> where T: Copy + Sub<Output=T> {
    pub(crate) fn width(&self) -> T {
        self.max.x - self.min.x
    }

    pub(crate) fn height(&self) -> T {
        self.max.y - self.min.y
    }
}

impl<T> Rect<T> where T: Copy + PartialOrd {}