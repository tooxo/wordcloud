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
    fn to_string(&self, offset: &Point<f32>) -> String;
    fn append_to_string(&self, offset: &Point<f32>, string: &mut String);
    fn length_estimation(&self) -> usize;
}

impl SVGPathCommand {
    pub(crate) fn to_string(&self, offset: &Point<f32>) -> String {
        match self {
            SVGPathCommand::Move(e) => e.to_string(offset),
            SVGPathCommand::Line(e) => e.to_string(offset),
            SVGPathCommand::QuadCurve(e) => e.to_string(offset),
            SVGPathCommand::Curve(e) => e.to_string(offset),
            SVGPathCommand::End(e) => e.to_string(offset),
        }
    }

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
pub(crate) struct Curve {
    pub(crate) t2: Point<f32>,
    pub(crate) t1: Point<f32>,
    pub(crate) t: Point<f32>,
    pub(crate) p_o: Point<f32>,
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
    fn to_string(&self, offset: &Point<f32>) -> String {
        format!("L {} {}", self.end.x + offset.x, self.end.y + offset.y)
    }

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
    fn to_string(&self, offset: &Point<f32>) -> String {
        format!(
            "M {} {}",
            self.position.x + offset.x,
            self.position.y + offset.y
        )
    }

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
    fn to_string(&self, offset: &Point<f32>) -> String {
        format!(
            "Q {} {}, {} {} ",
            self.t1.x + offset.x,
            self.t1.y + offset.y,
            self.t.x + offset.x,
            self.t.y + offset.y
        )
    }

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
    fn to_string(&self, offset: &Point<f32>) -> String {
        format!(
            "C {} {}, {} {}, {} {} ",
            self.t1.x + offset.x,
            self.t1.y + offset.y,
            self.t2.x + offset.x,
            self.t2.y + offset.y,
            self.t.x + offset.x,
            self.t.y + offset.y
        )
    }

    fn append_to_string(&self, _offset: &Point<f32>, _string: &mut String) {
        todo!()
    }

    fn length_estimation(&self) -> usize {
        todo!()
    }
}

impl SvgCommand for End {
    fn to_string(&self, _offset: &Point<f32>) -> String {
        String::from("Z")
    }

    fn append_to_string(&self, _offset: &Point<f32>, string: &mut String) {
        string.push('Z');
    }

    fn length_estimation(&self) -> usize {
        1
    }
}

impl QuadCurve {
    pub(crate) fn divide_curve(&self, parts: usize) -> Vec<Point<f32>> {
        let v = vec![parts; parts];
        v.iter()
            .enumerate()
            .map(|(i, &x)| (1.0 / (x as f32)) * i as f32)
            .map(|x| self.get_point_on_curve(x))
            .collect::<Vec<Point<f32>>>()
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
