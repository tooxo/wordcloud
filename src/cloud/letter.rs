use crate::common::path_collision::approximate_curve;
use crate::common::svg_command::{Curve, End, Line, Move, QuadCurve, SVGPathCommand};
use crate::types::point::Point;
use crate::types::rect::Rect;
use crate::types::rotation::Rotation;
use rusttype::OutlineBuilder;

#[derive(Debug)]
pub(crate) struct Letter {
    pub(crate) char: char,
    pub(crate) pixel_bounding_box: Rect<f32>,

    pub(crate) offset: Point<f32>,
    cursor: Point<f32>,
    pub(crate) state: Vec<SVGPathCommand>,
    pub(crate) simplified_state: Option<Vec<Line<f32>>>,
    pub(crate) rotation: Rotation,
}

impl Letter {
    pub fn new(
        char: char,
        pixel_bounding_box: Rect<f32>,
        offset: Point<f32>,
        rotation: Rotation,
    ) -> Self {
        Self {
            char,
            pixel_bounding_box,
            offset,
            cursor: Point::default(),
            state: Vec::default(),
            simplified_state: None,
            rotation,
        }
    }

    pub(crate) fn relative_bounding_box(&self, rotation: &Rotation) -> Rect<f32> {
        let bbox = self.pixel_bounding_box;

        rotation.rotate_rectangle(
            bbox + Point {
                x: 0.0,
                y: self.offset.y,
            },
        )
    }

    pub(crate) fn absolute_collidables(
        &self,
        rotation: &Rotation,
        word_offset: Point<f32>,
    ) -> Vec<Line<f32>> {
        let v = rotation.rotate_point(Point {
            x: self.offset.x,
            y: self.offset.y,
        });
        match &self.simplified_state {
            None => vec![],
            Some(b) => b
                .iter()
                .map(|y| Line {
                    start: y.start + v + word_offset,
                    end: y.end + v + word_offset,
                })
                .collect::<Vec<Line<f32>>>(),
        }
    }

    pub(crate) fn d(&self, global_off: &Point<f32>) -> String {
        let off: Point<f32> = self.rotation.rotate_point(self.offset) + *global_off;
        String::from_iter(self.state.iter().map(|x| x.to_string(&off)))
    }

    pub(crate) fn simplify(&mut self) {
        self.simplified_state = Some(
            self.state
                .iter()
                .flat_map(|x| match x {
                    SVGPathCommand::Move(_) => vec![],
                    SVGPathCommand::Line(l) => vec![*l],
                    SVGPathCommand::QuadCurve(q) => approximate_curve(q),
                    SVGPathCommand::Curve(_c) => unimplemented!("curve"),
                    SVGPathCommand::End(_) => vec![],
                })
                .collect::<Vec<Line<f32>>>(),
        );
    }
}

impl OutlineBuilder for Letter {
    fn move_to(&mut self, x: f32, y: f32) {
        self.cursor = self.rotation.rotate_point(Point { x, y });
        self.state.push(SVGPathCommand::Move(Move {
            position: self.rotation.rotate_point(Point { x, y }),
        }));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.state.push(SVGPathCommand::Line(Line {
            start: self.cursor,
            end: self.rotation.rotate_point(Point { x, y }),
        }));
        self.cursor = self.rotation.rotate_point(Point { x, y });
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.state.push(SVGPathCommand::QuadCurve(QuadCurve {
            t1: self.rotation.rotate_point(Point { x: x1, y: y1 }),
            t: self.rotation.rotate_point(Point { x, y }),
            p_o: self.cursor,
        }));
        self.cursor = self.rotation.rotate_point(Point { x, y });
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.state.push(SVGPathCommand::Curve(Curve {
            t2: self.rotation.rotate_point(Point { x: x2, y: y2 }),
            t1: self.rotation.rotate_point(Point { x: x1, y: y1 }),
            t: self.rotation.rotate_point(Point { x, y }),
            p_o: self.cursor,
        }));
        self.cursor = self.rotation.rotate_point(Point { x, y });
    }

    fn close(&mut self) {
        self.state.push(SVGPathCommand::End(End {}));
    }
}
