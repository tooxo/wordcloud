use crate::types::point::Point;
use num_traits::Zero;
use std::f32::consts::PI;
use std::ops::{Add, Deref, Sub};

#[derive(Debug, Clone)]
pub(crate) enum SVGPathCommand {
    Move(Move),
    Line(Line<f32>),
    QuadCurve(QuadCurve),
    Curve(Curve),
    End(End),
}

macro_rules! format_float {
    ($num:expr) => {
        format!("{:.2}", $num).deref()
    };
}

pub(crate) trait SvgCommand {
    fn append_to_string(&self, offset: &Point<f32>, string: &mut String);
    fn length_estimation(&self) -> usize;
}

impl SVGPathCommand {
    pub(crate) fn append_to_string(&self, offset: &Point<f32>, string: &mut String) {
        match self {
            SVGPathCommand::Move(x) => x.append_to_string(offset, string),
            SVGPathCommand::Line(x) => x.append_to_string(offset, string),
            SVGPathCommand::QuadCurve(x) => x.append_to_string(offset, string),
            SVGPathCommand::Curve(x) => x.append_to_string(offset, string),
            SVGPathCommand::End(x) => x.append_to_string(offset, string),
        }
    }

    pub(crate) fn length_estimation(&self) -> usize {
        match self {
            SVGPathCommand::Move(x) => x.length_estimation(),
            SVGPathCommand::Line(x) => x.length_estimation(),
            SVGPathCommand::QuadCurve(x) => x.length_estimation(),
            SVGPathCommand::Curve(x) => x.length_estimation(),
            SVGPathCommand::End(x) => x.length_estimation(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Line<T> {
    pub(crate) start: Point<T>,
    pub(crate) end: Point<T>,
}

#[allow(dead_code)]
pub(crate) struct Parallelogram<T> {
    p1: Point<T>,
    p2: Point<T>,
    p3: Point<T>,
    p4: Point<T>,
}

#[allow(dead_code)]
impl<T> Parallelogram<T> {
    pub(crate) fn lines(&self) -> Vec<Line<T>>
    where
        T: Copy,
    {
        vec![
            Line {
                start: self.p1,
                end: self.p2,
            },
            Line {
                start: self.p2,
                end: self.p3,
            },
            Line {
                start: self.p3,
                end: self.p4,
            },
            Line {
                start: self.p4,
                end: self.p1,
            },
        ]
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Move {
    pub(crate) position: Point<f32>,
}

#[derive(Debug, Clone)]
pub(crate) struct QuadCurve {
    pub(crate) t1: Point<f32>,
    pub(crate) t: Point<f32>,
    pub(crate) p_o: Point<f32>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
// https://www.geogebra.org/classic/WPHQ9rUt
pub(crate) struct Curve {
    pub(crate) c2: Point<f32>,
    pub(crate) c1: Point<f32>,
    pub(crate) e: Point<f32>,
    pub(crate) s: Point<f32>,
}

#[derive(Debug, Clone)]
pub(crate) struct End {}

#[allow(dead_code)]
impl<T> Line<T>
where
    T: Copy + Sub<Output = T> + Add<Output = T>,
{
    fn rotate_point(point: Point<T>, angle: T) -> Point<T>
    where
        T: num_traits::float::Float,
    {
        Point {
            x: point.x * angle.cos() - point.y * angle.sin(),
            y: point.y * angle.cos() + point.x * angle.sin(),
        }
    }

    pub(crate) fn thicken(&self, width: T) -> Parallelogram<T>
    where
        T: num_traits::float::Float + Zero + From<f32>,
    {
        let half_width = width / <T as From<f32>>::from(2.0);

        let height = (self.start.y - self.end.y).abs();
        let width = (self.start.x - self.end.x).abs();

        let deg = if height == T::zero() {
            <T as From<f32>>::from(PI) / <T as From<f32>>::from(2.0)
        } else if width == T::zero() {
            // line is straight
            T::zero()
        } else {
            let dy = self.start.y - self.end.y;
            let dx = self.start.x - self.end.x;

            (dy / dx).atan()
        };

        let start_rot = Line::rotate_point(self.start, deg);
        let end_rot = Line::rotate_point(self.end, deg);

        let vec = Point {
            x: half_width,
            y: <T as From<f32>>::from(0.0),
        };
        let (p1, p2, p3, p4) = (
            start_rot - vec,
            start_rot + vec,
            end_rot - vec,
            end_rot + vec,
        );

        Parallelogram {
            p1: Line::rotate_point(p1, -deg),
            p2: Line::rotate_point(p2, -deg),
            p3: Line::rotate_point(p3, -deg),
            p4: Line::rotate_point(p4, -deg),
        }
    }
}

impl SvgCommand for Line<f32> {
    fn append_to_string(&self, offset: &Point<f32>, string: &mut String) {
        string.push_str("L ");
        string.push_str(format_float!(self.end.x + offset.x));
        string.push(' ');
        string.push_str(format_float!(self.end.y + offset.y))
    }

    fn length_estimation(&self) -> usize {
        3 + 7 + 7
    }
}

impl SvgCommand for Move {
    fn append_to_string(&self, offset: &Point<f32>, string: &mut String) {
        string.push_str("M ");
        string.push_str(format_float!(self.position.x + offset.x));
        string.push(' ');
        string.push_str(format_float!(self.position.y + offset.y))
    }

    fn length_estimation(&self) -> usize {
        3 + 7 + 7
    }
}

impl SvgCommand for QuadCurve {
    fn append_to_string(&self, offset: &Point<f32>, string: &mut String) {
        string.push_str("Q ");
        string.push_str(format_float!(self.t1.x + offset.x));
        string.push(' ');
        string.push_str(format_float!(self.t1.y + offset.y));
        string.push(',');
        string.push_str(format_float!(self.t.x + offset.x));
        string.push(' ');
        string.push_str(format_float!(self.t.y + offset.y))
    }

    fn length_estimation(&self) -> usize {
        3 + 7 + 7 + 3 + 7 + 7
    }
}

impl SvgCommand for Curve {
    fn append_to_string(&self, offset: &Point<f32>, string: &mut String) {
        string.push_str("C ");
        string.push_str(format_float!(self.c2.x + offset.x));
        string.push(' ');
        string.push_str(format_float!(self.c2.y + offset.y));
        string.push(',');
        string.push_str(format_float!(self.c1.x + offset.x));
        string.push(' ');
        string.push_str(format_float!(self.c1.y + offset.y));
        string.push(',');
        string.push_str(format_float!(self.e.x + offset.x));
        string.push(' ');
        string.push_str(format_float!(self.e.y + offset.y));
        string.push(',');
    }

    fn length_estimation(&self) -> usize {
        2 + 6 + 7 + 7 + 7 + 7 + 7 + 7
    }
}

impl SvgCommand for End {
    fn append_to_string(&self, _offset: &Point<f32>, string: &mut String) {
        string.push('Z');
    }

    fn length_estimation(&self) -> usize {
        1
    }
}

impl QuadCurve {
    pub(crate) fn divide_quad(&self, center_points: usize) -> Vec<Point<f32>> {
        let points = center_points + 2;
        (0..points)
            .map(|i| (1. / (points - 1) as f32) * i as f32)
            .map(|x| self.get_point_on_curve(x))
            .collect()
    }

    /// See https://www.geogebra.org/m/YGqtDGzK
    fn get_point_on_curve(&self, t: f32) -> Point<f32> {
        // F = D + t(E - D)
        // D = A + t(B - A)
        // E = B + t(C - B)

        // A = p_o
        // B = t
        // C = t1

        let d = self.p_o + (self.t - self.p_o) * t;
        let e = self.t + (self.t1 - self.t) * t;

        d + (e - d) * t
    }
}

impl Curve {
    pub(crate) fn divide_curve(&self, center_points: usize) -> Vec<Point<f32>> {
        let points = center_points + 2;
        (0..points)
            .map(|i| (1. / (points - 1) as f32) * i as f32)
            .map(|x| self.get_point_on_curve(x))
            .collect()
    }

    fn get_point_on_curve(&self, t: f32) -> Point<f32> {
        /*
        p1 = s
        p2 = c1
        p3 = c2
        p4 = e

        P_5 = (1-t) P_1 + t P_2
        P_6 = (1-t) P_2 + t P_3
        P_7 = (1-t) P_3 + t P_4
        P_8 = (1-t) P_5 + t P_6
        P_9 = (1-t) P_6 + t P_7
        BZ = (1-t) P_8 +t P_9
         */

        let inv_t = 1. - t;
        let p5 = self.s * inv_t + self.c1 * t;
        let p6 = self.c1 * inv_t + self.c2 * t;
        let p7 = self.c2 * inv_t + self.e * t;
        let p8 = p5 * inv_t + p6 * t;
        let p9 = p6 * inv_t + p7 * t;

        p8 * inv_t + p9 * t
    }
}
