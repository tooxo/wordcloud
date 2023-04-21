use crate::common::svg_command::{Curve, Line, QuadCurve};
use crate::types::point::Point;
use itertools::Itertools;
use std::ops::{Mul, Sub};

fn ccw<T>(a: Point<T>, b: Point<T>, c: Point<T>) -> bool
where
    T: Copy + Sub<Output = T> + Mul<Output = T> + PartialOrd,
{
    (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
}

fn i<T>(a: Point<T>, b: Point<T>, c: Point<T>, d: Point<T>) -> bool
where
    T: Copy + Sub<Output = T> + Mul<Output = T> + PartialOrd,
{
    ccw(a, c, d) != ccw(b, c, d) && ccw(a, b, c) != ccw(a, b, d)
}

pub(crate) fn collide_line_line<T>(a: &Line<T>, b: &Line<T>) -> bool
where
    T: Copy + Sub<Output = T> + Mul<Output = T> + PartialOrd,
{
    i(a.start, a.end, b.start, b.end)
}

pub(crate) fn approximate_quad(a: &QuadCurve) -> Vec<Line<f32>> {
    let approx = a.divide_quad(1);
    approx
        .iter()
        .tuple_windows()
        .map(|(s, e)| Line { start: *s, end: *e })
        .collect()
}

pub(crate) fn approximate_curve(a: &Curve) -> Vec<Line<f32>> {
    let approx = a.divide_curve(2);
    approx
        .iter()
        .tuple_windows()
        .map(|(s, e)| Line { start: *s, end: *e })
        .collect()
}
