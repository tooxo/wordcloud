use crate::cloud::word::{Inp, Word};
use crate::common::font::FontSet;

use crate::image::{average_color_for_rect, canny_algorithm, Dimensions};
use crate::types::point::Point;
use crate::types::rect::Rect;
use crate::types::rotation::Rotation;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use image::imageops::grayscale;
use image::{DynamicImage, GenericImageView, Rgba};
use itertools::Itertools;
use parking_lot::{Mutex, RwLock};
use quadtree_rs::area::{Area, AreaBuilder};

use quadtree_rs::Quadtree;
use rand::thread_rng;
use rand::Rng;
use rayon::iter::ParallelIterator;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator};

use std::sync::Arc;

use svg::node::element::{Group, Path, Rectangle, Style, Text};
use svg::{Document, Node};

const QUADTREE_DIVISOR: f32 = 4.;

macro_rules! available_parallelism {
    () => {
        match std::thread::available_parallelism() {
            Ok(par) => usize::from(par),
            Err(_) => 4,
        }
    };
}

pub struct WordCloud<'a> {
    ct: RwLock<Quadtree<u64, Word<'a>>>,
    bg: Option<Quadtree<u64, ()>>,
    bg_image: Option<&'a DynamicImage>,
    dimensions: Dimensions,
    font: &'a FontSet<'a>,
}

impl<'a> WordCloud<'a> {
    fn needed_tree_depth(dimensions: Dimensions) -> f32 {
        ((dimensions.width().max(dimensions.height()) as f32 / QUADTREE_DIVISOR).log2()
            / 2.0_f32.log2())
        .ceil()
    }

    fn new(dimensions: Dimensions, font: &'a FontSet<'a>) -> Self {
        WordCloud {
            ct: RwLock::new(Quadtree::new(
                WordCloud::needed_tree_depth(dimensions) as usize
            )),
            bg: None,
            bg_image: None,
            dimensions,
            font,
        }
    }

    fn add_background(&mut self, image: &'a DynamicImage) {
        let resize = image.resize(
            (self.dimensions.width() as f32 / QUADTREE_DIVISOR) as u32,
            (self.dimensions.height() as f32 / QUADTREE_DIVISOR) as u32,
            image::imageops::FilterType::Nearest,
        );
        let grey = grayscale(&resize);
        let borders = canny_algorithm(&grey, 1.5);
        let border_image = borders.as_image();
        let mut qt = Quadtree::new(WordCloud::needed_tree_depth(self.dimensions) as usize);

        for (x, y, col) in border_image.pixels() {
            if col.0[0] != 0 || col.0[1] != 0 || col.0[2] != 0 {
                let (pos_x, pos_y) = (f32::max(x as f32 - 1., 0.), f32::max(y as f32 - 1., 0.));

                let search_area = AreaBuilder::default()
                    .anchor((pos_x as u64, pos_y as u64).into())
                    .dimensions(((4.) as u64, (4.) as u64))
                    .build()
                    .unwrap();

                let insert_area = AreaBuilder::default()
                    .anchor(((x as f32) as u64, (y as f32) as u64).into())
                    .dimensions(((1.) as u64, (1.) as u64))
                    .build()
                    .unwrap();

                let other = qt.query(search_area).next();
                if let Some(o) = other {
                    let comb = Rect::from(&insert_area).combine_rects(&Rect::from(&o.area()));
                    if let Some(com) = comb {
                        qt.delete_by_handle(o.handle());
                        qt.insert(Area::from(&com), ());
                        continue;
                    }
                }
                qt.insert(insert_area, ());
            }
        }

        self.bg = Some(qt);
        self.bg_image = Some(image);
    }

    fn converted_dimensions(&self) -> Rect<f32> {
        Rect {
            min: Point::default(),
            max: Point {
                x: self.dimensions.width() as f32,
                y: self.dimensions.height() as f32,
            },
        }
    }

    fn add_word(&self, mut word: Word<'a>) {
        let mut theta = 0.0_f64;
        let mut placed = false;
        let mut iters = 0;
        loop {
            if self.converted_dimensions().contains(&word.bounding_box) {
                let mut intersected: bool = false;

                let search_region = AreaBuilder::default()
                    .anchor(quadtree_rs::point::Point {
                        x: f32::max((word.bounding_box.min.x / QUADTREE_DIVISOR).ceil() - 1., 0.)
                            as u64,
                        y: f32::max((word.bounding_box.min.y / QUADTREE_DIVISOR).ceil() - 1., 0.)
                            as u64,
                    })
                    .dimensions((
                        (word.bounding_box.width() / QUADTREE_DIVISOR).ceil() as u64 + 2,
                        (word.bounding_box.height() / QUADTREE_DIVISOR).ceil() as u64 + 2,
                    ))
                    .build()
                    .unwrap();

                let insert_region = AreaBuilder::default()
                    .anchor(
                        (
                            (word.bounding_box.min.x / QUADTREE_DIVISOR).ceil() as u64,
                            (word.bounding_box.min.y / QUADTREE_DIVISOR).ceil() as u64,
                        )
                            .into(),
                    )
                    .dimensions((
                        (word.bounding_box.width() / QUADTREE_DIVISOR).ceil() as u64,
                        (word.bounding_box.height() / QUADTREE_DIVISOR).ceil() as u64,
                    ))
                    .build()
                    .unwrap();

                if let Some(qt_bg) = &self.bg {
                    if qt_bg.query(insert_region).next().is_some() {
                        intersected = true;
                    }
                }

                let mut len_bf = 0;
                if !intersected {
                    let read = self.ct.read();
                    len_bf = read.len();
                    for result in read.query(search_region) {
                        if word.word_intersect(result.value_ref()) {
                            intersected = true;
                            break;
                        }
                    }
                }

                if !intersected {
                    // println!("Placed {} {} {}", word.text, quad_tree.len(), iters);
                    placed = true;

                    let mut write = self.ct.write();
                    // read the newly added handles
                    for handle_id in len_bf..=write.len() {
                        if let Some(new_entry) = write.get(handle_id as u64) {
                            if word.word_intersect(new_entry.value_ref()) {
                                intersected = true;
                                break;
                            }
                        }
                    }
                    if !intersected {
                        match write.insert(insert_region, word) {
                            None => {
                                panic!("insertion failed");
                            }
                            Some(_) => {}
                        }
                        break;
                    }
                }
            }

            if iters % 10 == 0 && iters > 0 {
                let new_pos = Point {
                    x: thread_rng().gen_range(0.0..self.dimensions.width() as f32),
                    y: thread_rng().gen_range(0.0..self.dimensions.height() as f32),
                };

                iters += 1;

                word.move_word(&new_pos);
                theta = 0.0;
            } else {
                let revelations = theta / (std::f64::consts::PI * 2.0f64);
                if revelations < 5.0 {
                    theta += 1.0;
                } else {
                    theta += 0.1_f64;
                }

                const B: f64 = 5.0_f64;
                let r = B * theta;

                let new_pos = Point {
                    x: ((r * theta.cos()) as i32 + word.offset.x as i32) as f32,
                    y: ((r * theta.sin()) as i32 + word.offset.y as i32) as f32,
                };

                iters += 1;

                word.move_word(&new_pos);
            }
            if iters % 25 == 0 {
                // dbg!("resizing");
                if word.scale <= 10. {
                    break;
                }

                word = Word::build(
                    word.text.as_str(),
                    self.font,
                    word.scale - 2.,
                    word.offset,
                    if rand::random() {
                        Rotation::Zero
                    } else {
                        Rotation::TwoSeventy
                    },
                );
            }
            /*if iters > 55 {
                break;
            }*/
        }

        if !placed {
            // println!("Failed to place!");
        }
    }

    pub(crate) fn put_text_sync(&self, inp: Vec<Word<'a>>) {
        for word in inp {
            self.add_word(word);
        }
    }

    pub(crate) fn put_text(&self, inp: Vec<Word<'a>>) {
        let xl = (0..available_parallelism!())
            .map(|n| {
                inp.iter()
                    .skip(n)
                    .step_by(available_parallelism!())
                    .cloned()
                    .collect::<Vec<Word>>()
            })
            .collect::<Vec<Vec<Word>>>();

        xl.into_par_iter().for_each(|wl| self.put_text_sync(wl));
    }

    pub fn write_to_file(&self, filename: &str) {
        let ct = self.ct.read();
        let collected_entries: Vec<&Word> = ct.iter().map(|x| x.value_ref()).collect();

        let sliced = collected_entries.par_iter().chunks(
            (collected_entries.len() as f64 / available_parallelism!() as f64).ceil() as usize,
        );

        let doc_mutex = Arc::new(Mutex::new(
            Document::new()
                .set(
                    "viewBox",
                    (0, 0, self.dimensions.width(), self.dimensions.height()),
                )
                .set("height", self.dimensions.height())
                .set("width", self.dimensions.width()),
        ));

        let multiplier = match self.bg_image {
            None => 1.,
            Some(img) => {
                img.width() as f64
                    / usize::min(self.dimensions.width(), self.dimensions.height()) as f64
            }
        };

        sliced.for_each(|x| {
            for word in x {
                let color = match self.bg_image {
                    None => Rgba([0; 4]),
                    Some(img) => {
                        let integer_rect = Rect {
                            min: Point {
                                x: ((word.bounding_box.min.x as f64) * multiplier) as u32,
                                y: ((word.bounding_box.min.y as f64) * multiplier) as u32,
                            },
                            max: Point {
                                x: ((word.bounding_box.max.x as f64) * multiplier) as u32,
                                y: ((word.bounding_box.max.y as f64) * multiplier) as u32,
                            },
                        };

                        average_color_for_rect(img, &integer_rect, Rgba([0, 0, 0, 0]))
                    }
                };

                let p = Path::new()
                    .set("d", word.d())
                    .set("stoke", "none")
                    .set("fill", crate::image::color_to_rgb_string(&color));
                let _s = p.to_string();
                {
                    doc_mutex.lock().append(p);
                }
            }
        });
        svg::save(filename, &doc_mutex.lock().clone()).unwrap();
    }
}

impl<'a> WordCloud<'a> {
    fn debug_background_collision(&self, filename: &str) {
        let mut document = Document::new()
            .set(
                "viewBox",
                (0, 0, self.dimensions.width(), self.dimensions.height()),
            )
            .set("height", self.dimensions.height())
            .set("width", self.dimensions.width());

        let colors = vec![
            "black", "gray", "silver", "maroon", "red", "purple", "fuchsia", "green", "lime",
            "olive", "yellow", "navy", "blue", "teal", "aqua",
        ];
        if let Some(i) = self.bg.as_ref() {
            for bound in i.iter() {
                let random_color = colors[thread_rng().gen_range(0..colors.len())];

                let rec = Rectangle::new()
                    .set("x", bound.anchor().x as f32 * QUADTREE_DIVISOR)
                    .set("y", bound.anchor().y as f32 * QUADTREE_DIVISOR)
                    .set("width", bound.area().width() as f32 * QUADTREE_DIVISOR)
                    .set("height", bound.area().height() as f32 * QUADTREE_DIVISOR)
                    .set("stroke", "black")
                    .set("stroke-width", "1px")
                    .set("fill", random_color);

                document.append(rec);
            }
        }

        svg::save(filename, &document).unwrap();
    }
    fn debug_result_on_background(&self, filename: &str) {
        let mut document = Document::new()
            .set(
                "viewBox",
                (0, 0, self.dimensions.width(), self.dimensions.height()),
            )
            .set("height", self.dimensions.height())
            .set("width", self.dimensions.width());

        if let Some(i) = self.bg.as_ref() {
            for bound in i.iter() {
                let rec = Rectangle::new()
                    .set("x", bound.anchor().x as f32 * QUADTREE_DIVISOR)
                    .set("y", bound.anchor().y as f32 * QUADTREE_DIVISOR)
                    .set("width", bound.area().width() as f32 * QUADTREE_DIVISOR)
                    .set("height", bound.area().height() as f32 * QUADTREE_DIVISOR);

                document.append(rec);
            }
        }

        for word in self.ct.read().iter() {
            let p = Path::new()
                .set("d", word.value_ref().d())
                .set("stoke", "none")
                .set("fill", "gray");
            document.append(p);
        }

        svg::save(filename, &document).unwrap();
    }

    fn debug_collidables(&self, filename: &str) {
        let mut document = Document::new()
            .set(
                "viewBox",
                (0, 0, self.dimensions.width(), self.dimensions.height()),
            )
            .set("height", self.dimensions.height())
            .set("width", self.dimensions.width());

        for x in self.ct.read().iter() {
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

    fn debug_text(&self, filename: &str) {
        let mut document = Document::new()
            .set(
                "viewBox",
                (0, 0, self.dimensions.width(), self.dimensions.height()),
            )
            .set("height", self.dimensions.height())
            .set("width", self.dimensions.width());

        let read_lock = self.ct.read();
        for (font, group) in &read_lock
            .iter()
            .map(|y| y.value_ref())
            .group_by(|k| k.used_font)
        {
            let dt = match font.packed() {
                None => font.reference().data,
                Some(s) => s.as_slice(),
            };
            let enc = STANDARD_NO_PAD.encode(dt);
            document.append(Style::new(format!(
                "@font-face{{font-family:\"{}\";src:url(\"data:{};charset=utf-8;base64,{}\");}}",
                font.name(),
                font.font_type().embed_tag(),
                enc
            )));

            let mut gr = Group::new().set("font-family", font.name());

            for word in group {
                let mut t = Text::new()
                    .set("x", word.offset.x)
                    .set("y", word.offset.y)
                    .set("font-size", word.scale);
                match word.rotation {
                    Rotation::Zero => (),
                    Rotation::Ninety | Rotation::OneEighty | Rotation::TwoSeventy => {
                        t.assign(
                            "style",
                            format!(
                                "transform: rotate({}deg); transform-origin: {}px {}px",
                                word.rotation.inner(),
                                word.offset.x,
                                word.offset.y
                            ),
                        );
                    }
                }
                t.append(svg::node::Text::new(&word.text));

                gr.append(t);
            }

            document.append(gr);
        }

        svg::save(filename, &document).unwrap();
    }
}

/**
Builder for [WordCloud]
 */
#[derive(Default)]
pub struct WordCloudBuilder<'a> {
    dimensions: Option<Dimensions>,
    font: Option<&'a FontSet<'a>>,
    image: Option<&'a DynamicImage>,
}

impl<'a> WordCloudBuilder<'a> {
    pub fn new() -> Self {
        WordCloudBuilder::default()
    }

    /**
    Output dimensions of the created image
     */
    pub fn dimensions(mut self, dimensions: Dimensions) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    /**
    Used [`FontSet`], see [`FontSet`] for more information
     */
    pub fn font(mut self, font: &'a FontSet<'a>) -> Self {
        self.font = Some(font);
        self
    }

    /**
    Optional: Image, which is used for border detection
     */
    pub fn image(mut self, image: &'a DynamicImage) -> Self {
        self.image = Some(image);
        self
    }

    /**
    Build the [`WordCloud`], basically free, no calculations are done here
     */
    pub fn build(self) -> Result<WordCloud<'a>, String> {
        let mut wc = match (self.dimensions, self.font) {
            (Some(d), Some(f)) => WordCloud::new(d, f),
            (_, None) => return Err("Missing FontSet in WordCloudBuilder!".into()),
            (None, _) => return Err("Missing Dimensions in WordCloudBuilder!".into()),
        };

        if let Some(i) = self.image {
            wc.add_background(i);
        }

        Ok(wc)
    }
}

pub(crate) fn create_word_cloud(
    dimensions: Dimensions,
    font: FontSet,
    inp: Vec<Inp>,
    background_image: &DynamicImage,
) {
    let mut words = (0..available_parallelism!())
        .into_par_iter()
        .map(|n| {
            inp.iter()
                .skip(n)
                .step_by(available_parallelism!())
                .collect::<Vec<&Inp>>()
        })
        .map(|inputs| {
            inputs.into_iter().map(|x| {
                let left_offs = thread_rng().gen_range(0.0..dimensions.width() as f32);
                let top_offs = thread_rng().gen_range(0.0..dimensions.height() as f32);
                Word::build(
                    &x.text,
                    &font,
                    x.scale,
                    Point {
                        x: left_offs,
                        y: top_offs,
                    },
                    x.rotation,
                )
            })
        })
        .flatten_iter()
        .collect::<Vec<Word>>();

    words.sort_by_key(|x| x.scale as usize);
    words.reverse();

    let em: &[Word] = &[];

    let (first, second) = if words.len() > 20 {
        words.split_at(20)
    } else {
        (words.as_slice(), em)
    };

    let wc = WordCloudBuilder::new()
        .dimensions(dimensions)
        .font(&font)
        .image(background_image)
        .build()
        .unwrap();

    wc.put_text_sync(first.to_vec());
    wc.put_text(second.to_vec());
    wc.write_to_file("created.svg");

    if true {
        println!("Dumping Debug Files");

        if !std::path::Path::new("debug").is_dir() {
            std::fs::create_dir("debug").unwrap();
        }
        if wc.bg.is_some() {
            wc.debug_background_collision("debug/background_collision.svg");
            wc.debug_result_on_background("debug/result_on_background.svg");
        }
        wc.debug_collidables("debug/collidables.svg");
        wc.debug_text("debug/text.svg");
    }
}
