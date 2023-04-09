use crate::common::svg_command::{Curve, Line, QuadCurve};
use crate::types::point::Point;
use itertools::Itertools;

fn ccw(a: Point<f32>, b: Point<f32>, c: Point<f32>) -> bool {
    (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
}

fn i(a: Point<f32>, b: Point<f32>, c: Point<f32>, d: Point<f32>) -> bool {
    ccw(a, c, d) != ccw(b, c, d) && ccw(a, b, c) != ccw(a, b, d)
}

pub(crate) fn collide_line_line(a: &Line<f32>, b: &Line<f32>) -> bool {
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
