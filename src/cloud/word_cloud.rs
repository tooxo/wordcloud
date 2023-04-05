use crate::cloud::word::Word;
use crate::cloud::Inp;
use crate::common::font::Font;
use crate::image::{average_color_for_rect, canny_algorithm, Dimensions};
use crate::io::debug::{
    debug_background_collision, debug_background_on_result, debug_collidables, debug_text,
};
use crate::types::point::Point;
use crate::types::rect::Rect;
use image::imageops::grayscale;
use image::{DynamicImage, GenericImageView, Rgba};
use parking_lot::{Mutex, RwLock};
use quadtree_rs::area::{Area, AreaBuilder};
use quadtree_rs::entry::Entry;
use quadtree_rs::Quadtree;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rayon::iter::IndexedParallelIterator;
use rayon::iter::ParallelIterator;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};

use std::sync::Arc;
use svg::node::element::{Path};
use svg::{Document, Node};

const QUADTREE_DIVISOR: f32 = 4.;
const PROCESSING_SLICES: usize = 8; /*match std::thread::available_parallelism() {
                                        Ok(par) => usize::from(par),
                                        Err(_) => 4,
                                    };*/

struct WordCloud<'a> {
    ct: Arc<RwLock<Quadtree<u64, Word>>>,
    bg: Option<Quadtree<u64, ()>>,
    bg_image: Option<&'a DynamicImage>,
    dimensions: Dimensions,
    random: Mutex<SmallRng>,
    font: Font<'a>,
}

impl<'a> WordCloud<'a> {
    fn needed_tree_depth(dimensions: Dimensions) -> f32 {
        ((usize::max(dimensions.width(), dimensions.height()) as f32 / QUADTREE_DIVISOR).log2()
            / 2.0_f32.log2())
        .ceil()
    }

    pub(crate) fn new(dimensions: Dimensions, font: Font<'a>) -> Self {
        WordCloud {
            ct: Arc::new(RwLock::new(Quadtree::new(
                WordCloud::needed_tree_depth(dimensions) as usize,
            ))),
            bg: None,
            bg_image: None,
            dimensions,
            random: Mutex::new(SmallRng::from_entropy()),
            font,
        }
    }

    pub(crate) fn add_background(&mut self, image: &'a DynamicImage) {
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

    fn add_word(&self, mut word: Word) {
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
                        if word.word_intersect(result.value_ref()).is_some() {
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
                            if word.word_intersect(new_entry.value_ref()).is_some() {
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
                let mut lck = self.random.lock();
                let new_pos = Point {
                    x: lck.gen_range(0.0..self.dimensions.width() as f32),
                    y: lck.gen_range(0.0..self.dimensions.height() as f32),
                };

                drop(lck);

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
                    self.font.clone(),
                    word.scale - 5.,
                    word.offset,
                    word.rotation,
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

    pub(crate) fn put_text_sync(&self, inp: Vec<Word>) {
        for word in inp {
            self.add_word(word);
        }
    }
    pub(crate) fn put_text(&mut self, inp: Vec<Word>) {
        let xl = (0..PROCESSING_SLICES)
            .map(|n| {
                inp.iter()
                    .skip(n)
                    .step_by(PROCESSING_SLICES)
                    .cloned()
                    .collect::<Vec<Word>>()
            })
            .collect::<Vec<Vec<Word>>>();

        xl.into_par_iter().for_each(|wl| self.put_text_sync(wl));
    }

    pub(crate) fn write_to_file(&self, filename: &str) {
        let ct = self.ct.read();
        let collected_entries: Vec<&Word> = ct.iter().map(|x| x.value_ref()).collect();

        let sliced = collected_entries
            .par_iter()
            .chunks((collected_entries.len() as f64 / PROCESSING_SLICES as f64).ceil() as usize);

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

pub(crate) fn create_word_cloud(
    dimensions: Dimensions,
    font: Font,
    inp: Vec<Inp>,
    background_image: &DynamicImage,
) {
    let random = Arc::new(Mutex::new(SmallRng::from_entropy()));

    let slices = (0..PROCESSING_SLICES)
        .map(|n| {
            inp.iter()
                .skip(n)
                .step_by(PROCESSING_SLICES)
                .collect::<Vec<&Inp>>()
        })
        .collect::<Vec<Vec<&Inp>>>();

    let words = slices
        .into_par_iter()
        .map(|inps| {
            let fnt = font.clone();
            let cl = random.clone();
            inps.into_iter().map(move |x| {
                let mut locked = cl.lock();
                let left_offs = locked.gen_range(0.0..dimensions.width() as f32);
                let top_offs = locked.gen_range(0.0..dimensions.height() as f32);
                drop(locked);
                Word::build(
                    &x.text,
                    fnt.clone(),
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

    let mut wc = WordCloud::new(dimensions, font.clone());
    wc.add_background(background_image);
    wc.put_text(words);
    wc.write_to_file("created.svg");

    if true {
        println!("Dumping Debug Files");
        let ct = wc.ct.read();
        let entries: Vec<&Entry<u64, Word>> = ct.iter().collect();

        if !std::path::Path::new("debug").is_dir() {
            std::fs::create_dir("debug").unwrap();
        }
        if let Some(bg) = wc.bg {
            debug_background_collision(
                "debug/background_collision.svg",
                bg.iter().collect::<Vec<&Entry<u64, ()>>>(),
                QUADTREE_DIVISOR,
                dimensions,
            );
            debug_background_on_result(
                "debug/text_on_background.svg",
                &entries,
                &bg.iter().collect::<Vec<&Entry<u64, ()>>>(),
                QUADTREE_DIVISOR,
                dimensions,
            );
        }
        debug_collidables("debug/collidables.svg", &entries, dimensions);
        debug_text("debug/text.svg", &entries, dimensions, &font);
    }
}
