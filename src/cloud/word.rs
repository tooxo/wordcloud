use std::slice::Iter;
use rusttype::{Font, Scale};
use crate::cloud::letter::Letter;
use crate::common::path_collision::collide_line_line;
use crate::common::svg_command::Line;
use crate::types::point::Point;
use crate::types::rect::Rect;
use crate::types::rotation::Rotation;

pub(crate) struct Word {
    pub(crate) text: String,
    pub(crate) glyphs: Vec<Letter>,
    pub(crate) offset: Point<f32>,

    pub(crate) bounding_box: Rect<f32>,
    pub(crate) scale: Scale,
    pub(crate) rotation: Rotation,
}


impl<'font> Word {
    pub(crate) fn build(text: &str, font: &'font Font<'font>, scale: Scale, start: Point<f32>, rotation: Rotation) -> Word {
        let gl = font.layout(text, scale, rusttype::Point::default());
        let glyphs = gl
            .into_iter()
            .enumerate()
            .map(
                |(i, x)| {
                    let offset = Point::from(&x.position());
                    let bbox = x.pixel_bounding_box().unwrap();
                    let mut letter = Letter::new(
                        text.chars().collect::<Vec<char>>()[i],
                        Rect {
                            min: Point {
                                x: bbox.min.x as f32,
                                y: bbox.min.y as f32,
                            },
                            max: Point {
                                x: bbox.max.x as f32,
                                y: bbox.max.y as f32,
                            },
                        },
                        offset,
                        rotation,
                    );
                    x.unpositioned().build_outline(&mut letter);
                    letter.simplify();
                    letter
                }
            ).collect::<Vec<Letter>>();

        let mut w = Word { text: String::from(text), glyphs, offset: start, bounding_box: Rect::default(), scale, rotation };
        w.recalculate_bounding_box();

        w
    }

    fn recalculate_bounding_box(&mut self) {
        let last_glyph = self.glyphs.last().unwrap();
        let first_glyph = self.glyphs.first().unwrap();
        let mut max_y = 0.0;
        let mut min_y = f32::MAX;

        for glyph in &self.glyphs {
            let bbox = glyph.pixel_bounding_box;
            if bbox.max.y > max_y {
                max_y = bbox.max.y;
            }
            if bbox.min.y < min_y {
                min_y = bbox.min.y;
            }
        }

        let base_rect: Rect<f32> = Rect {
            min: Point {
                x: first_glyph.pixel_bounding_box.min.x,
                y: min_y,
            },
            max: Point {
                x: last_glyph.pixel_bounding_box.max.x,
                y: max_y,
            },
        };

        let rotated = self.rotation.rotate_rectangle(base_rect);
        self.bounding_box = Rect {
            min: rotated.min + self.offset,
            max: rotated.max + self.offset,
        };
    }

    pub(crate) fn move_word(&mut self, new_position: &Point<f32>) {
        self.offset = Point { x: new_position.x, y: new_position.y };
        self.recalculate_bounding_box();
    }

    fn collidables(&self) -> impl Iterator<Item=Line<f32>> + '_ {
        self.glyphs
            .iter()
            .flat_map(
                |x| x.absolute_collidables(&self.rotation, self.offset)
            )
    }

    pub(crate) fn word_intersect_(&self, other: &Word) -> Option<Point<f32>> {
        if self.bounding_box.max.x < other.bounding_box.min.x || self.bounding_box.max.y < other.bounding_box.min.y {
            return None;
        }
        if self.bounding_box.min.x > other.bounding_box.max.x || self.bounding_box.min.y > other.bounding_box.max.y {
            return None;
        }

        let other_collidables = other.collidables();
        let line1 = Line {
            start: self.bounding_box.min,
            end: Point {
                x: self.bounding_box.max.x,
                y: self.bounding_box.min.y,
            },
        };
        let line2 = Line {
            start: Point {
                x: self.bounding_box.max.x,
                y: self.bounding_box.min.y,
            },
            end: self.bounding_box.max,
        };
        let line3 = Line {
            start: self.bounding_box.max,
            end: Point {
                x: self.bounding_box.min.x,
                y: self.bounding_box.max.y,
            },
        };
        let line4 = Line {
            start: Point {
                x: self.bounding_box.min.x,
                y: self.bounding_box.max.y,
            },
            end: self.bounding_box.min,
        };

        let lines = vec![line1, line2, line3, line4];
        for coll in other_collidables {
            for line in &lines {
                let o = collide_line_line(line, &coll);

                if let Some(i) = o {
                    return Some(i);
                }
            }
        }
        None
    }

    pub(crate) fn word_intersect_a(&self, other: &Word) -> Option<Point<f32>> {
        if self.bounding_box.max.x < other.bounding_box.min.x || self.bounding_box.max.y < other.bounding_box.min.y {
            return None;
        }
        if self.bounding_box.min.x > other.bounding_box.max.x || self.bounding_box.min.y > other.bounding_box.max.y {
            return None;
        }

        let left_collidables = self.collidables();
        let right_collidables = other.collidables().collect::<Vec<Line<f32>>>();

        for left_collidable in left_collidables {
            for right_collidable in &right_collidables {
                let o = collide_line_line(&left_collidable, right_collidable);

                if let Some(i) = o {
                    return Some(i);
                }
            }
        }
        None
    }

    pub(crate) fn word_intersect(&self, other: &Word) -> Option<Point<f32>> {
        if !self.bounding_box.overlaps(&other.bounding_box) {
            return None;
        }

        let right_collidables = other.collidables();

        // check if contained inside another word

        if other.bounding_box.contains(&self.bounding_box) {
            // word completely is inside another word

            let midpoint = Point {
                x: self.bounding_box.min.x + self.bounding_box.width() / 2.0,
                y: self.bounding_box.min.y + self.bounding_box.height() / 2.0,
            };

            // construct a line out of the containing word
            let line = Line {
                start: midpoint,
                end: Point {
                    x: midpoint.x,
                    y: other.bounding_box.min.y - 10.0,
                },
            };

            let mut collisions = 0;
            for collidable in right_collidables {
                if collide_line_line(&collidable, &line).is_some() {
                    collisions += 1;
                }
            }

            if self.text == "melt" {
                dbg!(collisions);
                dbg!(midpoint);
                dbg!(&self.bounding_box);
                dbg!(&self.offset);
            }

            if collisions % 2 == 1 {
                // its inside the text
                return Some(
                    Point::default()
                );
            }
        }


        // let own_collidables = self.collidables().collect::<Vec<Line<f32>>>();
        for glyph in &other.glyphs {
            let bbox = glyph.relative_bounding_box(&other.rotation) + other.offset;
            if self.bounding_box.overlaps(&bbox) {
                for l in glyph.absolute_collidables(&other.rotation, other.offset) {
                    // let thick = l.thicken(10.0);
                    // let lines = thick.lines();

                    if self.bounding_box.extend(5.0).intersects(&l) {
                        return Some(Point::default());
                    }

                    /*for own in &own_collidables {
                        for line in &lines {
                            let o = collide_line_line(line, own);

                            if let Some(i) = o {
                                return Some(i);
                            }
                        }
                    }*/
                }
            }
        }

        /*for left_collidable in &left_collidables {
            let thick = left_collidable.thicken(10.0);
            let thick_lines = thick.lines();
            for right_collidable in &right_collidables {
                for thick_line in &thick_lines {
                    let o = collide_line_line(right_collidable, thick_line);

                    if let Some(i) = o {
                        return Some(i);
                    }
                }
            }
        }*/
        if self.text == "melt" {
            dbg!("placed");
        }

        None
    }
}
