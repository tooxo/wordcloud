use std::cell::RefCell;
use swash::scale::outline::Outline;
use swash::scale::ScaleContext;
use swash::shape::Direction::LeftToRight;
use swash::shape::ShapeContext;

use swash::zeno::Command;
use swash::zeno::PathData;

use crate::cloud::letter::Letter;
use crate::common::font::{Font, FontSet, GuessScript};
use crate::common::svg_command::{Curve, End, Line, Move, QuadCurve, SVGPathCommand};
use crate::types::point::Point;
use crate::types::rect::Rect;
use crate::types::rotation::Rotation;

thread_local! {
    static SCALER_TL: RefCell<ScaleContext> = RefCell::new(ScaleContext::new());
    static SHAPER_TL: RefCell<ShapeContext> = RefCell::new(ShapeContext::new());
}

type WordBuildingError = String;
type WordBuildingResult<T> = Result<T, WordBuildingError>;

#[derive(Clone)]
pub(crate) struct Word<'a> {
    pub(crate) text: String,
    pub(crate) glyphs: Vec<Letter>,
    pub(crate) offset: Point<f32>,

    pub(crate) bounding_box: Rect<f32>,
    pub(crate) scale: f32,
    pub(crate) rotation: Rotation,
    pub(crate) used_font: &'a Font<'a>,
}

impl<'a> Word<'a> {
    pub(crate) fn build(
        text: &str,
        font: &'a FontSet,
        font_size: f32,
        start: Point<f32>,
        rotation: Rotation,
    ) -> WordBuildingResult<Word<'a>> {
        let ws = text.guess_script();
        let used_font = match font.get_font_for_script(&ws) {
            None => return Err(format!("No font found, which supports: \"{}\"", text)),
            Some(f) => f,
        };

        let mut w = SHAPER_TL.with(|shape_ref| {
            SCALER_TL.with(|scale_ref| {
                let mut shape_context = shape_ref.borrow_mut();
                let mut shaper = shape_context
                    .builder(*used_font.reference())
                    .script(ws.s())
                    .size(font_size)
                    .direction(LeftToRight)
                    .features(&[
                        ("lnum", 1),
                        ("tnum", 1),
                        ("pnum", 1),
                        ("ordn", 1),
                        ("swsh", 1),
                        ("liga", 1),
                        ("hlig", 1),
                        ("dlig", 1),
                        ("calt", 1),
                    ])
                    .build();

                let mut scale_context = scale_ref.borrow_mut();
                let mut scaler = scale_context
                    .builder(*used_font.reference())
                    .size(font_size)
                    .build();

                shaper.add_str(text);

                let mut letters = Vec::new();
                let chars = text.chars().collect::<Vec<char>>();
                let mut char_index = 0;
                let mut advance = 0.0;
                shaper.shape_with(|c| {
                    letters.append(
                        &mut c
                            .glyphs
                            .iter()
                            .map(|glyph| {
                                let outline =
                                    scaler.scale_outline(glyph.id).unwrap_or(Outline::new());

                                let bounds = outline.bounds();
                                let bbox = Rect {
                                    min: Point {
                                        x: bounds.min.x + advance,
                                        y: bounds.min.y,
                                    },
                                    max: Point {
                                        x: bounds.max.x + advance,
                                        y: bounds.max.y,
                                    },
                                };

                                let mut letter = Letter::new(
                                    chars[char_index],
                                    bbox,
                                    Point {
                                        x: glyph.x + advance,
                                        y: glyph.y,
                                    },
                                    rotation,
                                );

                                let commands = outline.path();
                                for command in commands.commands() {
                                    let cmd: Command = command;
                                    match cmd {
                                        Command::MoveTo(p) => letter.move_to(p.x, p.y),
                                        Command::LineTo(p) => letter.line_to(p.x, p.y),
                                        Command::CurveTo(x1, x2, x) => {
                                            letter.curve_to(x1.x, x1.y, x2.x, x2.y, x.x, x.y)
                                        }
                                        Command::QuadTo(x1, x) => {
                                            letter.quad_to(x1.x, x1.y, x.x, x.y)
                                        }
                                        Command::Close => letter.close(),
                                    }
                                }
                                advance += glyph.advance;
                                char_index += 1;

                                letter
                            })
                            .collect::<Vec<Letter>>(),
                    );
                });

                Word {
                    text: String::from(text),
                    glyphs: letters,
                    offset: start,
                    bounding_box: Rect::default(),
                    scale: font_size,
                    rotation,
                    used_font,
                }
            })
        });

        let mut max_y = 0.0;
        let mut min_y = f32::MAX;
        for x in &w.glyphs {
            if x.pixel_bounding_box.max.y > max_y {
                max_y = x.pixel_bounding_box.max.y;
            }
            if x.pixel_bounding_box.min.y < min_y {
                min_y = x.pixel_bounding_box.min.y;
            }
        }

        for glyph in &mut w.glyphs {
            let height_pt = Point::default();
            for command in &mut glyph.state {
                *command = match &command {
                    SVGPathCommand::Move(p) => SVGPathCommand::Move(Move {
                        position: glyph.rotation.rotate_point(
                            height_pt.sub_ly(&glyph.rotation.rotate_point_back(&p.position)),
                        ),
                    }),
                    SVGPathCommand::Line(p) => SVGPathCommand::Line(Line {
                        start: glyph.rotation.rotate_point(
                            height_pt.sub_ly(&glyph.rotation.rotate_point_back(&p.start)),
                        ),
                        end: glyph.rotation.rotate_point(
                            height_pt.sub_ly(&glyph.rotation.rotate_point_back(&p.end)),
                        ),
                    }),
                    SVGPathCommand::QuadCurve(q) => SVGPathCommand::QuadCurve(QuadCurve {
                        s: glyph.rotation.rotate_point(
                            height_pt.sub_ly(&glyph.rotation.rotate_point_back(&q.s)),
                        ),
                        e: glyph.rotation.rotate_point(
                            height_pt.sub_ly(&glyph.rotation.rotate_point_back(&q.e)),
                        ),
                        c1: glyph.rotation.rotate_point(
                            height_pt.sub_ly(&glyph.rotation.rotate_point_back(&q.c1)),
                        ),
                    }),
                    SVGPathCommand::Curve(c) => SVGPathCommand::Curve(Curve {
                        c2: glyph.rotation.rotate_point(
                            height_pt.sub_ly(&glyph.rotation.rotate_point_back(&c.c2)),
                        ),
                        c1: glyph.rotation.rotate_point(
                            height_pt.sub_ly(&glyph.rotation.rotate_point_back(&c.c1)),
                        ),
                        e: glyph.rotation.rotate_point(
                            height_pt.sub_ly(&glyph.rotation.rotate_point_back(&c.e)),
                        ),
                        s: glyph.rotation.rotate_point(
                            height_pt.sub_ly(&glyph.rotation.rotate_point_back(&c.s)),
                        ),
                    }),
                    SVGPathCommand::End(_) => SVGPathCommand::End(End {}),
                }
            }

            glyph.pixel_bounding_box = Rect {
                min: height_pt.sub_ly(&glyph.pixel_bounding_box.min),
                max: height_pt.sub_ly(&glyph.pixel_bounding_box.max),
            };
            glyph.pixel_bounding_box.normalize();
            glyph.simplify();
            assert!(glyph.pixel_bounding_box.is_normal());
        }

        w.recalculate_bounding_box();
        assert!(w.bounding_box.is_normal());
        assert_ne!(w.bounding_box.width(), 0.);
        assert_ne!(w.bounding_box.height(), 0.);

        Ok(w)
    }

    fn recalculate_bounding_box(&mut self) {
        if self.glyphs.is_empty() {
            return;
        }

        #[allow(clippy::unwrap_used)]
        let last_glyph = self.glyphs.last().unwrap();
        #[allow(clippy::unwrap_used)]
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
        self.offset = Point {
            x: new_position.x,
            y: new_position.y,
        };
        self.recalculate_bounding_box();
    }

    fn collidables(&self) -> impl Iterator<Item = Line<f32>> + '_ {
        self.glyphs
            .iter()
            .flat_map(|x| x.absolute_collidables(&self.rotation, self.offset))
    }

    pub(crate) fn d(&self) -> String {
        self.glyphs.iter().map(|g| g.d(&self.offset)).collect()
    }

    pub(crate) fn word_intersect(&self, other: &Word) -> bool {
        if !self.bounding_box.extend(5.0).overlaps(&other.bounding_box) {
            return false;
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
            let high_line = Line {
                start: midpoint,
                end: (midpoint.x, other.bounding_box.min.y - 10.0).into(),
            };

            let low_line = Line {
                start: midpoint,
                end: (midpoint.x, other.bounding_box.max.y + 10.0).into(),
            };

            let (mut col_h, mut col_l) = (0, 0);
            for collidable in right_collidables {
                if collidable.intersects(&high_line) {
                    col_h += 1;
                }
                if collidable.intersects(&low_line) {
                    col_l += 1;
                }
            }

            if col_h % 2 == 1 || col_l % 2 == 1 {
                // its inside the text
                return true;
            }
        }

        let extended = self.bounding_box.extend(2.0);
        for glyph in &other.glyphs {
            if extended.overlaps(&(glyph.relative_bounding_box(&other.rotation) + other.offset)) {
                for l in &glyph.absolute_collidables(&other.rotation, other.offset) {
                    if extended.intersects(l) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[derive(Default)]
pub(crate) struct WordBuilder<'a> {
    content: Option<String>,
    scale: Option<f32>,
    font: Option<&'a FontSet<'a>>,
}

impl<'a> WordBuilder<'a> {
    pub(crate) fn new() -> Self {
        WordBuilder::default()
    }

    pub(crate) fn content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }
    pub(crate) fn scale(mut self, scale: f32) -> Self {
        self.scale = Some(scale);
        self
    }
    pub(crate) fn font(mut self, font: &'a FontSet<'a>) -> Self {
        self.font = Some(font);
        self
    }

    pub(crate) fn build(&self) -> WordBuildingResult<Word<'a>> {
        #[allow(clippy::unwrap_used)]
        Word::build(
            self.content.as_ref().unwrap(),
            self.font.unwrap(),
            self.scale.unwrap(),
            Point::default(),
            Rotation::Zero,
        )
    }
}
