use crate::common::svg_command::{Curve, Line, QuadCurve};
use crate::types::point::Point;

pub(crate) enum Collidable {
    Line(Line<f32>),
    BezierQuad(QuadCurve),
    EllipticCurve(Curve),
}

impl Collidable {
    pub(crate) fn to_absolute(&self, offset: Point<f32>) -> Collidable {
        match self {
            Collidable::Line(l) => Collidable::Line(Line {
                start: l.start + offset,
                end: l.end + offset,
            }),
            Collidable::BezierQuad(bq) => Collidable::BezierQuad(QuadCurve {
                t1: bq.t1 + offset,
                t: bq.t + offset,
                p_o: bq.p_o + offset,
            }),
            Collidable::EllipticCurve(_ec) => {
                unimplemented!("elliptic curve");
            }
        }
    }
}

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

pub(crate) fn collide_line_line_(a: &Line<f32>, b: &Line<f32>) -> Option<Point<f32>> {
    let u_a: f32 = ((b.end.x - b.start.x) * (a.start.y - b.start.y)
        - (b.end.y - b.start.y) * (a.start.x - b.start.x))
        / ((b.end.y - b.start.y) * (a.end.x - a.start.x)
            - (b.end.x - b.start.x) * (a.end.x - a.start.x));
    let u_b: f32 = ((a.end.x - a.start.x) * (a.start.y - b.start.y)
        - (a.end.y - a.start.y) * (a.start.x - b.start.x))
        / ((b.end.y - b.start.y) * (a.end.x - a.start.x)
            - (b.end.x - b.start.x) * (a.end.x - a.start.x));

    if (0.0..=1.).contains(&u_a) && (0.0..=1.).contains(&u_b) {
        let int_x = a.start.x + (u_a * (a.end.x - a.start.x));
        let int_y = a.start.y + (u_a * (a.end.y - a.start.x));

        return Some(Point { x: int_x, y: int_y });
    }
    None
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

fn collide_line_bezier(a: &Line<f32>, b: &QuadCurve) -> Option<Point<f32>> {
    let lines = approximate_curve(b);
    let colission = lines
        .iter()
        .map(|l| collide_line_line(l, &a))
        .filter(|x| x.is_some())
        .collect::<Vec<Option<Point<f32>>>>();

    if colission.is_empty() {
        None
    } else {
        *colission.first().unwrap()
    }
}

fn collide_bezier_bezier(a: &QuadCurve, b: &QuadCurve) -> Option<Point<f32>> {
    let apx_a = approximate_curve(a);

    let collission = apx_a
        .iter()
        .map(|l| collide_line_bezier(l, b))
        .filter(|x| x.is_some())
        .collect::<Vec<Option<Point<f32>>>>();

    if collission.is_empty() {
        None
    } else {
        *collission.first().unwrap()
    }
}

pub(crate) fn find_collision_spot(a: &Collidable, b: &Collidable) -> Option<Point<f32>> {
    match a {
        Collidable::Line(la) => match b {
            Collidable::Line(lb) => collide_line_line(&la, &lb),
            Collidable::BezierQuad(qc) => collide_line_bezier(&la, &qc),
            Collidable::EllipticCurve(_) => unimplemented!("elliptic"),
        },
        Collidable::BezierQuad(qc) => match b {
            Collidable::Line(la) => collide_line_bezier(&la, &qc),
            Collidable::BezierQuad(qb) => collide_bezier_bezier(&qc, &qb),
            Collidable::EllipticCurve(_) => unimplemented!("elliptic"),
        },
        Collidable::EllipticCurve(_) => unimplemented!("elliptic curve"),
    }
}
