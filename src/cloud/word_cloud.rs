use crate::cloud::word::Word;
use crate::cloud::Inp;
use crate::common::font::Font;
use crate::image::image::{average_color_for_rect, canny_algorithm, Dimensions};
use crate::io::debug::{
    debug_background_collision, debug_background_on_result, debug_collidables, debug_text,
};
use crate::types::point::Point;
use crate::types::rect::Rect;
use image::imageops::grayscale;
use image::{DynamicImage, GenericImageView, Rgba};
use itertools::Itertools;
use parking_lot::{Mutex, RwLock};
use quadtree_rs::area::{Area, AreaBuilder};
use quadtree_rs::entry::Entry;
use quadtree_rs::Quadtree;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rayon::iter::ParallelIterator;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};

use std::sync::Arc;
use svg::node::element::Path;
use svg::{Document, Node};

fn fill_background_qt(
    qt: &mut Quadtree<u64, ()>,
    image: &DynamicImage,
    output_image_dims: Dimensions,
    quadtree_divisor: f32,
) {
    let resize = image.resize(
        (output_image_dims.width() as f32 / quadtree_divisor) as u32,
        (output_image_dims.height() as f32 / quadtree_divisor) as u32,
        image::imageops::FilterType::Nearest,
    );
    let grey = grayscale(&resize);
    let borders = canny_algorithm(&grey, 1.5);
    let border_image = borders.as_image();

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
}

pub(crate) fn create_word_cloud(
    dimensions: Dimensions,
    font: Font,
    inp: Vec<Inp>,
    background_image: &DynamicImage,
) {
    const QUADTREE_DIVISOR: f32 = 4.;
    const A: f64 = 0.5_f64;
    const B: f64 = 5.0f64;

    let random_arc = Arc::new(Mutex::new(SmallRng::from_seed([1; 32])));

    let depth = ((usize::max(dimensions.width(), dimensions.height()) as f32 / QUADTREE_DIVISOR)
        .log2()
        / 2.0_f32.log2())
    .ceil() as usize;
    let qt_content: Quadtree<u64, Word> = Quadtree::new(depth);
    let mut qt_background: Quadtree<u64, ()> = Quadtree::new(depth);

    fill_background_qt(
        &mut qt_background,
        background_image,
        dimensions,
        QUADTREE_DIVISOR,
    );

    let processing_slices = match std::thread::available_parallelism() {
        Ok(par) => usize::from(par),
        Err(_) => 4,
    };

    let slices = inp
        .as_slice()
        .splitn(
            (inp.len() as f64 / processing_slices as f64).ceil() as usize,
            |_| false,
        )
        .collect::<Vec<&[Inp]>>();

    let words = slices
        .into_par_iter()
        .map(|inps| {
            let fnt = font.clone();
            let cl = random_arc.clone();
            inps.iter().map(move |x| {
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

    let visible_space = Rect {
        min: Point { x: 0.0, y: 0.0 },
        max: Point {
            x: dimensions.width() as f32,
            y: dimensions.height() as f32,
        },
    };

    let qt_content_lock = Arc::new(RwLock::new(qt_content));
    let chu = (0..processing_slices)
        .map(|n| {
            words
                .iter()
                .skip(n)
                .step_by(processing_slices)
                .cloned()
                .collect::<Vec<Word>>()
        })
        .collect::<Vec<Vec<Word>>>();

    let cloned_lock = qt_content_lock.clone();
    chu.into_par_iter().for_each(|words| {
        for mut word in words {
            let mut theta = 0.0_f64;
            let mut placed = false;
            let mut iters = 0;
            loop {
                if visible_space.contains(&word.bounding_box) {
                    let mut intersected: bool = false;

                    let search_region = AreaBuilder::default()
                        .anchor(quadtree_rs::point::Point {
                            x: f32::max(
                                (word.bounding_box.min.x / QUADTREE_DIVISOR).ceil() - 1.,
                                0.,
                            ) as u64,
                            y: f32::max(
                                (word.bounding_box.min.y / QUADTREE_DIVISOR).ceil() - 1.,
                                0.,
                            ) as u64,
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

                    if qt_background.query(insert_region).next().is_some() {
                        intersected = true;
                    }

                    let mut len_bf = 0;
                    if !intersected {
                        let read = cloned_lock.read();
                        len_bf = read.len();
                        for result in read.query(search_region) {
                            let intersection = word.word_intersect(result.value_ref());
                            if intersection.is_some() {
                                intersected = true;
                                break;
                            }
                        }
                    }

                    if !intersected {
                        // println!("Placed {} {} {}", word.text, quad_tree.len(), iters);
                        placed = true;

                        let mut write = cloned_lock.write();
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
                    let mut lck = random_arc.lock();
                    let new_pos = Point {
                        x: lck.gen_range(0.0..dimensions.width() as f32),
                        y: lck.gen_range(0.0..dimensions.height() as f32),
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

                    let r = A + B * theta;

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
                        font.clone(),
                        word.scale - 5.,
                        word.offset,
                        word.rotation,
                    );
                }
                if iters > 55 {
                    break;
                }
            }

            if !placed {
                // println!("Failed to place!");
            }
        }
    });

    let qt_content = qt_content_lock.read();
    let entries: Vec<&Entry<u64, Word>> = qt_content.iter().collect();
    let collected_entries: Vec<&Word> = qt_content.iter().map(|x| x.value_ref()).collect();

    let sliced = collected_entries
        .as_slice()
        .splitn(
            (collected_entries.len() as f64 / processing_slices as f64).ceil() as usize,
            |_| false,
        )
        .collect::<Vec<&[&Word]>>();

    let doc_mutex = Arc::new(Mutex::new(
        Document::new()
            .set("viewBox", (0, 0, dimensions.width(), dimensions.height()))
            .set("height", dimensions.height())
            .set("width", dimensions.width()),
    ));

    let multiplier = background_image.width() as f64
        / usize::min(dimensions.width(), dimensions.height()) as f64;
    sliced.par_iter().for_each(|x| {
        for word in *x {
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
            let avg_color =
                average_color_for_rect(background_image, &integer_rect, Rgba([0, 0, 0, 0]));
            let p = Path::new()
                .set("d", word.d())
                .set("stoke", "none")
                .set("fill", crate::image::image::color_to_rgb_string(&avg_color));
            let _s = p.to_string();
            {
                doc_mutex.lock().append(p);
            }
        }
    });
    svg::save("created.svg", &doc_mutex.lock().clone()).unwrap();

    if true {
        println!("Dumping Debug Files");
        if !std::path::Path::new("debug").is_dir() {
            std::fs::create_dir("debug").unwrap();
        }
        debug_background_collision(
            "debug/background_collision.svg",
            qt_background.iter().collect::<Vec<&Entry<u64, ()>>>(),
            QUADTREE_DIVISOR,
            dimensions,
        );
        debug_collidables("debug/collidables.svg", &entries, dimensions);
        debug_text("debug/text.svg", &entries, dimensions);
        debug_background_on_result(
            "debug/text_on_background.svg",
            &entries,
            &qt_background.iter().collect::<Vec<&Entry<u64, ()>>>(),
            QUADTREE_DIVISOR,
            dimensions,
        );
    }
}
