use crate::cloud::word::{Word, WordBuilder};
use crate::common::font::FontSet;
use std::io::Cursor;
use std::io::Error;

#[cfg(feature = "background_image")]
use crate::image::{average_color_for_rect, canny_algorithm, color_to_rgb_string};
use crate::types::point::Point;
use crate::types::rect::Rect;
use crate::types::rotation::Rotation;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;

#[cfg(feature = "background_image")]
use image::imageops::grayscale;
#[cfg(feature = "background_image")]
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

use crate::font::GuessScript;
use crate::rank::RankedWords;
use crate::types::spiral::Spiral;
use crate::Dimensions;
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

#[cfg(not(feature = "background_image"))]
type DynamicImage = ();

/**
    Creates the WordCloud
*/
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

    #[cfg(feature = "background_image")]
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
                    .expect("Error while calculating dimensions");

                let insert_area = AreaBuilder::default()
                    .anchor(((x as f32) as u64, (y as f32) as u64).into())
                    .dimensions(((1.) as u64, (1.) as u64))
                    .build()
                    .expect("Error while calculating dimensions");

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
        let mut spiral = Spiral::new(5.);
        let mut iters = 0;

        let mut break_flag = false;
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
                    .expect("search region undefined");

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
                    .expect("insert region undefined");

                if let Some(qt_bg) = &self.bg {
                    if qt_bg.query(insert_region).next().is_some() {
                        intersected = true;
                    }
                }

                let len_bf = if !intersected {
                    let read = self.ct.read();

                    for result in read.query(search_region) {
                        if word.word_intersect(result.value_ref()) {
                            intersected = true;
                            break;
                        }
                    }
                    read.len()
                } else {
                    0
                };

                if !intersected {
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
            } else {
                println!(
                    "missed: {} {:?} {:?}",
                    iters,
                    word.normalized_bbox(),
                    word.bounding_box
                );
            }

            spiral.advance();
            let incoming_pos = spiral.position() + word.offset;
            let ranges = word.get_positioning_range(&self.dimensions);

            if iters % 10 == 0
                || !ranges.0.contains(&incoming_pos.x)
                || !ranges.1.contains(&incoming_pos.y)
            {
                let new_pos = Point {
                    x: thread_rng().gen_range(ranges.0),
                    y: thread_rng().gen_range(ranges.1),
                };

                iters += 1;

                word.move_word(&new_pos);

                spiral.reset();
            } else {
                let pos = spiral.position() + word.offset;

                iters += 1;

                word.move_word(&pos);

                // assert!(word.bounding_box.min.full_ge(&Point::default()));
                // assert!(ranged.0.contains(&word.bounding_box.min.x));
                // assert!(ranged.1.contains(&word.bounding_box.min.y));
            }
            if iters % 25 == 0 && iters != 0 {
                if word.scale <= 10. {
                    if break_flag {
                        // println!("Warning: missed word: {}", word.text);
                        break;
                    }
                    break_flag = true;
                } else {
                    word = match Word::build(
                        word.text.as_str(),
                        word.used_font,
                        word.scale - 5.,
                        word.offset,
                        Rotation::random(),
                    ) {
                        Ok(mut w) => {
                            if !self.converted_dimensions().contains(&w.bounding_box) {
                                let (xr, yr) = w.get_positioning_range(&self.dimensions);
                                let point1 = Point {
                                    x: thread_rng().gen_range(xr.clone()),
                                    y: thread_rng().gen_range(yr.clone()),
                                };
                                w.move_word(&point1);

                                assert!(self.converted_dimensions().contains(&w.bounding_box));
                            }

                            w
                        }
                        Err(_) => continue,
                    };
                }
            }
        }
    }

    pub(crate) fn put_text_sync(&self, inp: Vec<Word<'a>>) {
        for word in inp {
            self.add_word(word);
        }
    }

    pub(crate) fn put_text(&self, inp: Vec<Word<'a>>) {
        /*let xl = (0..available_parallelism!())
            .map(|n| {
                inp.iter()
                    .skip(n)
                    .step_by(available_parallelism!())
                    .cloned()
                    .collect::<Vec<Word>>()
            })
            .collect::<Vec<Vec<Word>>>();*/

        inp
            .into_par_iter()
            .for_each(
                |w| self.add_word(w)
            );

        // xl.into_par_iter().for_each(|wl| self.put_text_sync(wl));
    }

    /**
        Add new words to the [`WordCloud`]. For the best results, call this function only once.
    */
    pub fn write_content(&self, content: RankedWords, max_word_count: usize) {
        let max = content
            .0
            .iter()
            .take(max_word_count)
            .map(|x| (x.count() as f32))
            .sum::<f32>()
            / max_word_count as f32;

        let max = content.0.iter().max_by_key(|x| x.count()).unwrap().count() as f32;

        let inp: Vec<WordBuilder> = content
            .0
            .iter()
            .take(max_word_count)
            .flat_map(|w| {
                let font_size_range = Word::guess_font_size_range(w.content(), &self.dimensions);
                let ws = w.content().guess_script();
                let used_font = match self.font.get_font_for_script(&ws) {
                    None => {
                        return None;
                    }
                    Some(f) => f,
                };

                let scale = ((w.count() as f32).log2() / max.log2()) * font_size_range.end;
                Some(
                    WordBuilder::new()
                        .content(w.content().to_string())
                        .scale(scale)
                        .font(used_font)
                        .start(Point::default()),
                )
            })
            .collect();

        let mut words = (0..available_parallelism!())
            .into_par_iter()
            .map(|n| {
                inp.iter()
                    .skip(n)
                    .step_by(available_parallelism!())
                    .collect::<Vec<&WordBuilder>>()
            })
            .map(|inputs| inputs.into_iter().map(|x| x.build()))
            .flatten_iter()
            .flat_map(|pw| match pw {
                Ok(w) => Some(w),
                Err(e) => {
                    eprintln!("Warning: {}", e);
                    None
                }
            })
            .map(|mut w| {
                let (x_range, y_range) = w.get_positioning_range(&self.dimensions);

                let point = (
                    thread_rng().gen_range(x_range),
                    thread_rng().gen_range(y_range),
                );
                w.move_word(&point.into());

                w
            })
            .collect::<Vec<Word>>();

        words.sort_by_key(|d| d.scale as u64);
        words.reverse();

        let em: &[Word] = &[];
        let (first, second) = if words.len() > 20 {
            words.split_at(20)
        } else {
            (words.as_slice(), em)
        };

        self.put_text_sync(first.to_vec());
        self.put_text(second.to_vec());
    }

    #[cfg(feature = "background_image")]
    fn get_color_for_word(&self, word: &Word) -> Rgba<u8> {
        match self.bg_image {
            None => Rgba([0; 4]),
            Some(img) => {
                let multiplier = img.width() as f64
                    / usize::min(self.dimensions.width(), self.dimensions.height()) as f64;

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
        }
    }

    /**
        Export the resulting WordCloud as an SVG formatted [`String`]. Here the text is rendered using SVG Paths instead
        of Text elements. This leads to way bigger file sizes, but also to a little bit more accurate
        drawing of the text.

        To export using text elements, use the [`Self::export_text`]
        function.
    */
    pub fn export_rendered(&self) -> Result<String, Error> {
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

        sliced.for_each(|x| {
            for word in x {
                let mut p = Path::new().set("d", word.d()).set("stoke", "none");
                #[cfg(feature = "background_image")]
                {
                    let color = self.get_color_for_word(word);
                    p.assign("fill", color_to_rgb_string(&color));
                }

                let _s = p.to_string();
                {
                    doc_mutex.lock().append(p);
                }
            }
        });

        let lock = doc_mutex.lock();
        let mut target = Cursor::new(Vec::new());
        match svg::write(&mut target, &lock.clone()) {
            Ok(_) => {}
            Err(e) => return Err(e),
        };

        Ok(String::from_utf8(target.into_inner()).expect("decoding the written string failed"))
    }

    /**
        Writes the result of [`Self::export_rendered`] to a file.
    */
    pub fn export_rendered_to_file(&self, filename: &str) -> Result<(), Error> {
        let string = self.export_rendered()?;
        std::fs::write(filename, string.as_bytes())?;
        Ok(())
    }

    /**
       Export the resulting WordCloud as an SVG formatted [`String`]. Here the text is rendered
       using Text elements.

       This function should be preferred over [`Self::export_rendered`] in
       most use-cases.
    */
    pub fn export_text(&self) -> Result<String, Error> {
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

                #[cfg(feature = "background_image")]
                {
                    let color = self.get_color_for_word(word);
                    t.assign("fill", color_to_rgb_string(&color));
                }

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

        let mut cursor = Cursor::new(Vec::new());
        svg::write(&mut cursor, &document)?;

        Ok(String::from_utf8_lossy(&cursor.into_inner()).into())
    }

    /**
    Writes the result of [`Self::export_text`] to a file.
     */
    pub fn export_text_to_file(&self, filename: &str) -> Result<(), Error> {
        let string = self.export_text()?;
        std::fs::write(filename, string.as_bytes())?;
        Ok(())
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

        svg::save(filename, &document).expect("writing to file failed");
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

        svg::save(filename, &document).expect("writing to file failed");
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

        svg::save(filename, &document).expect("error exporting to file");
    }

    /**
        Exports versions of the WordCloud to a folder, which are mainly used for debugging purposes.
        This function may panic, so it shouldn't be used in production.
    */
    pub fn export_debug_to_folder(&self, folder_name: &str) {
        let fol = if folder_name.ends_with('/') {
            String::from(folder_name)
        } else {
            String::from(folder_name) + "/"
        };
        if !std::path::Path::new(&fol).is_dir() {
            std::fs::create_dir(&fol).expect("creating debug folder failed");
        }
        if self.bg.is_some() {
            self.debug_background_collision(&(fol.clone() + "background_collision.svg"));
            self.debug_result_on_background(&(fol.clone() + "result_on_background.svg"));
        }
        self.debug_collidables(&(fol + "collidables.svg"));
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

        #[cfg(feature = "background_image")]
        if let Some(i) = self.image {
            wc.add_background(i);
        }

        Ok(wc)
    }
}
