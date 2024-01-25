use std::ops::{Add, Mul, Sub};

#[derive(Copy, Clone, Debug)]
pub(crate) struct Point<T> {
    pub(crate) x: T,
    pub(crate) y: T,
}

impl<T> Add<Point<T>> for Point<T>
where
    T: Add<Output = T>,
{
    type Output = Point<T>;

    fn add(self, rhs: Point<T>) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Sub<Point<T>> for Point<T>
where
    T: Sub<Output = T>,
{
    type Output = Point<T>;

    fn sub(self, rhs: Point<T>) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> Point<T>
where
    T: Sub<Output = T> + Copy,
{
    #[allow(dead_code)]
    pub(crate) fn sub_lx(&self, other: &Point<T>) -> Point<T> {
        Point {
            x: self.x - other.x,
            y: other.y,
        }
    }
    pub(crate) fn sub_ly(&self, other: &Point<T>) -> Point<T> {
        Point {
            x: other.x,
            y: self.y - other.y,
        }
    }
}

impl<T> Mul<T> for Point<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Point<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> PartialEq<Self> for Point<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.x.eq(&other.x) && self.y.eq(&other.y)
    }
}

impl<T> Default for Point<T>
where
    T: Default,
{
    fn default() -> Self {
        Point {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<T> From<Point<T>> for quadtree_rs::point::Point<T> {
    fn from(value: Point<T>) -> Self {
        quadtree_rs::point::Point {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<&swash::zeno::Point> for Point<f32> {
    fn from(value: &swash::zeno::Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl<T> Point<T>
where
    T: PartialOrd,
{
    pub(crate) fn full_le(&self, other: &Point<T>) -> bool {
        self.x <= other.x && self.y <= other.y
    }

    #[allow(dead_code)]
    pub(crate) fn full_ge(&self, other: &Point<T>) -> bool {
        self.x >= other.x && self.y >= other.y
    }

    pub(crate) fn min(&self, other: &Point<T>) -> Point<T>
    where
        T: Copy,
    {
        Point {
            x: if self.x < other.x { self.x } else { other.x },
            y: if self.y < other.y { self.y } else { other.y },
        }
    }

    pub(crate) fn max(&self, other: &Point<T>) -> Point<T>
    where
        T: Copy,
    {
        Point {
            x: if self.x > other.x { self.x } else { other.x },
            y: if self.y > other.y { self.y } else { other.y },
        }
    }
}

impl<T> From<(T, T)> for Point<T> {
    fn from(value: (T, T)) -> Self {
        Point {
            x: value.0,
            y: value.1,
        }
    }
}

impl<T> From<quadtree_rs::point::Point<T>> for Point<T> {
    fn from(value: quadtree_rs::point::Point<T>) -> Self {
        (value.x, value.y).into()
    }
}
