use crate::common::path_collision::Collidable;
use crate::types::point::Point;
use num_traits::{Num, NumCast, Zero};
use std::f32::consts::PI;
use std::ops::{Add, Sub};

#[derive(Debug)]
pub(crate) enum SVGPathCommand {
    Move(Move),
    Line(Line<f32>),
    QuadCurve(QuadCurve),
    Curve(Curve),
    End(End),
}

pub(crate) trait SvgCommand {
    fn collidable(&self) -> Option<Collidable>;
    fn to_string(&self, offset: &Point<f32>) -> String;
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
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Line<T> {
    pub(crate) start: Point<T>,
    pub(crate) end: Point<T>,
}

pub(crate) struct Parallelogram<T> {
    p1: Point<T>,
    p2: Point<T>,
    p3: Point<T>,
    p4: Point<T>,
}

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

#[derive(Debug)]
pub(crate) struct Move {
    pub(crate) position: Point<f32>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct QuadCurve {
    pub(crate) t1: Point<f32>,
    pub(crate) t: Point<f32>,
    pub(crate) p_o: Point<f32>,
}

#[derive(Debug)]
pub(crate) struct Curve {
    pub(crate) t2: Point<f32>,
    pub(crate) t1: Point<f32>,
    pub(crate) t: Point<f32>,
    pub(crate) p_o: Point<f32>,
}

#[derive(Debug)]
pub(crate) struct End {}

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
    fn collidable(&self) -> Option<Collidable> {
        Some(Collidable::Line(*self))
    }

    fn to_string(&self, offset: &Point<f32>) -> String {
        format!("L {} {}", self.end.x + offset.x, self.end.y + offset.y)
    }
}

impl SvgCommand for Move {
    fn collidable(&self) -> Option<Collidable> {
        None
    }

    fn to_string(&self, offset: &Point<f32>) -> String {
        format!(
            "M {} {}",
            self.position.x + offset.x,
            self.position.y + offset.y
        )
    }
}

impl SvgCommand for QuadCurve {
    fn collidable(&self) -> Option<Collidable> {
        Some(Collidable::BezierQuad(*self))
    }

    fn to_string(&self, offset: &Point<f32>) -> String {
        format!(
            "Q {} {}, {} {} ",
            self.t1.x + offset.x,
            self.t1.y + offset.y,
            self.t.x + offset.x,
            self.t.y + offset.y
        )

        /*let mut s = String::new();
        let parts: Vec<Point<f32>> = self.divide_curve(2);
        for part in parts {
            s += &*format!(
                "L {} {}",
                part.x + f32::from(offset.x),
                part.y + f32::from(offset.y),
            );
        }
        s*/
    }
}

impl SvgCommand for Curve {
    fn collidable(&self) -> Option<Collidable> {
        unimplemented!("curve unimplemented")
    }

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
}

impl SvgCommand for End {
    fn collidable(&self) -> Option<Collidable> {
        None
    }

    fn to_string(&self, _offset: &Point<f32>) -> String {
        String::from("Z")
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
