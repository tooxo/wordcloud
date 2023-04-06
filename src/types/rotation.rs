use crate::types::point::Point;
use crate::types::rect::Rect;

#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub(crate) enum Rotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

impl Rotation {
    pub(crate) fn inner(&self) -> i64 {
        match self {
            Rotation::Zero => 0,
            Rotation::Ninety => 90,
            Rotation::OneEighty => 180,
            Rotation::TwoSeventy => 270,
        }
    }

    pub(crate) fn rotate_point(&self, point: Point<f32>) -> Point<f32> {
        match self {
            Rotation::Zero => point,
            Rotation::Ninety => Point {
                x: -point.y,
                y: point.x,
            },
            Rotation::OneEighty => Point {
                x: -point.x,
                y: -point.y,
            },
            Rotation::TwoSeventy => Point {
                x: point.y,
                y: -point.x,
            },
        }
    }

    pub(crate) fn rotate_point_back(&self, point: &Point<f32>) -> Point<f32> {
        match self {
            Rotation::Zero => *point,
            Rotation::Ninety => Point {
                x: point.y,
                y: -point.x,
            },
            Rotation::OneEighty => Point {
                x: -point.x,
                y: -point.y,
            },
            Rotation::TwoSeventy => Point {
                x: -point.y,
                y: point.x,
            },
        }
    }

    pub(crate) fn rotate_rectangle(&self, rect: Rect<f32>) -> Rect<f32> {
        let (rot_min, rot_max) = (self.rotate_point(rect.min), self.rotate_point(rect.max));
        Rect {
            min: Point {
                x: rot_min.x.min(rot_max.x),
                y: rot_min.y.min(rot_max.y),
            },
            max: Point {
                x: rot_min.x.max(rot_max.x),
                y: rot_min.y.max(rot_max.y),
            },
        }
    }
}
