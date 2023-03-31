use crate::common::svg_command::{Line, QuadCurve};
use crate::types::point::Point;

fn ccw(a: Point<f32>, b: Point<f32>, c: Point<f32>) -> bool {
    (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
}

fn i(a: Point<f32>, b: Point<f32>, c: Point<f32>, d: Point<f32>) -> bool {
    ccw(a, c, d) != ccw(b, c, d) && ccw(a, b, c) != ccw(a, b, d)
}

pub(crate) fn collide_line_line(a: &Line<f32>, b: &Line<f32>) -> Option<Point<f32>> {
    if !i(a.start, a.end, b.start, b.end) {
        None
    } else {
        Some(Point::default())
    }
}

pub(crate) fn approximate_curve(a: &QuadCurve) -> Vec<Line<f32>> {
    let apx = a.divide_curve(1);
    let mut last = a.p_o;
    let mut lines = vec![];
    for p in apx {
        lines.push(Line {
            start: last,
            end: p,
        });
        last = p;
    }

    lines.push(Line {
        start: last,
        end: a.t,
    });

    lines
}
