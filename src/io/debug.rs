use crate::cloud::word::Word;
use crate::common::font::Font;
use crate::image::Dimensions;
use base64::engine::general_purpose;
use base64::Engine;
use quadtree_rs::entry::Entry;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use svg::node::element::{Path, Rectangle, Style, Text};
use svg::{Document, Node};

pub(crate) fn debug_background_collision(
    filename: &str,
    qt_entries: Vec<&Entry<u64, ()>>,
    quadtree_divisor: f32,
    dimensions: Dimensions,
) {
    let mut random = SmallRng::from_entropy();
    let mut document = Document::new()
        .set("viewBox", (0, 0, dimensions.width(), dimensions.height()))
        .set("height", dimensions.height())
        .set("width", dimensions.width());

    let colors = vec![
        "black", "gray", "silver", "maroon", "red", "purple", "fushsia", "green", "lime", "olive",
        "yellow", "navy", "blue", "teal", "aqua",
    ];
    for bound in qt_entries {
        let random_color = colors[random.gen_range(0..colors.len())];

        let rec = Rectangle::new()
            .set("x", bound.anchor().x as f32 * quadtree_divisor)
            .set("y", bound.anchor().y as f32 * quadtree_divisor)
            .set("width", bound.area().width() as f32 * quadtree_divisor)
            .set("height", bound.area().height() as f32 * quadtree_divisor)
            .set("stroke", "black")
            .set("stroke-width", "1px")
            .set("fill", random_color);

        document.append(rec);
    }

    svg::save(filename, &document).unwrap();
}

pub(crate) fn debug_collidables(
    filename: &str,
    qt_entries: &Vec<&Entry<u64, Word>>,
    dimensions: Dimensions,
) {
    let mut document = Document::new()
        .set("viewBox", (0, 0, dimensions.width(), dimensions.height()))
        .set("height", dimensions.height())
        .set("width", dimensions.width());

    for x in qt_entries {
        let w = x.value_ref();
        for glyph in &w.glyphs {
            for x in glyph.absolute_collidables(&w.rotation, w.offset) {
                let p = Path::new()
                    .set("stroke", "black")
                    .set("stroke-width", 1)
                    .set(
                        "d",
                        format!("M {} {} L {} {} Z", x.start.x, x.start.y, x.end.x, x.end.y),
                    );
                document.append(p);
            }

            let r = glyph.relative_bounding_box(&w.rotation) + w.offset;
            let p = Rectangle::new()
                .set("stroke", "green")
                .set("stroke-width", 1)
                .set("fill", "none")
                .set("x", r.min.x)
                .set("y", r.min.y)
                .set("width", r.width())
                .set("height", r.height());

            document.append(p);
        }

        document.append(
            Rectangle::new()
                .set("stroke", "red")
                .set("stroke-width", 1)
                .set("fill", "none")
                .set("x", w.bounding_box.min.x)
                .set("y", w.bounding_box.min.y)
                .set("width", w.bounding_box.width())
                .set("height", w.bounding_box.height()),
        )
    }

    svg::save(filename, &document).unwrap();
}

pub(crate) fn debug_text(
    filename: &str,
    entries: &Vec<&Entry<u64, Word>>,
    dimensions: Dimensions,
    font: &Font,
) {
    let mut document = Document::new()
        .set("viewBox", (0, 0, dimensions.width(), dimensions.height()))
        .set("height", dimensions.height())
        .set("width", dimensions.width());

    let dt = font.re().data;
    let enc = general_purpose::STANDARD_NO_PAD.encode(dt);
    let style = Style::new(format!(
        "@font-face{{\
            font-family: \"Test\";\
            src: url(\"data:application/font-ttf;charset=utf-8;base64,{}\");\
            }}",
        enc
    ));

    document.append(style);

    for entry in entries {
        let word = entry.value_ref();
        let mut t = Text::new()
            .set("x", word.offset.x)
            .set("y", word.offset.y)
            .set("font-family", "Test")
            .set("font-size", word.scale);
        if word.rotation == crate::types::rotation::Rotation::Ninety {
            t.assign(
                "style",
                format!(
                    "transform: rotate(90deg); transform-origin: {}px {}px",
                    word.offset.x, word.offset.y
                ),
            );
        }
        t.append(svg::node::Text::new(&word.text));

        document.append(t);
    }

    svg::save(filename, &document).unwrap();
}

pub(crate) fn debug_background_on_result(
    filename: &str,
    entries: &Vec<&Entry<u64, Word>>,
    boundaries: &Vec<&Entry<u64, ()>>,
    quadtree_divisor: f32,
    dimensions: Dimensions,
) {
    let mut document = Document::new()
        .set("viewBox", (0, 0, dimensions.width(), dimensions.height()))
        .set("height", dimensions.height())
        .set("width", dimensions.width());

    for bound in boundaries {
        let rec = Rectangle::new()
            .set("x", bound.anchor().x as f32 * quadtree_divisor)
            .set("y", bound.anchor().y as f32 * quadtree_divisor)
            .set("width", bound.area().width() as f32 * quadtree_divisor)
            .set("height", bound.area().height() as f32 * quadtree_divisor);

        document.append(rec);
    }

    for word in entries {
        let p = Path::new()
            .set("d", word.value_ref().d())
            .set("stoke", "none")
            .set("fill", "gray");
        document.append(p);
    }

    svg::save(filename, &document).unwrap();
}
