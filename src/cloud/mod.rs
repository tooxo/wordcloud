use std::fs::create_dir;
use crate::cloud::word::Word;
use crate::types::point::Point;
use crate::types::rect::Rect;
use crate::types::rotation::Rotation;
use parking_lot::Mutex;
use quadtree_rs::area::{Area, AreaBuilder};

use quadtree_rs::Quadtree;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rayon::prelude::IntoParallelIterator;

use std::sync::Arc;

use svg::node::element::{Path, Rectangle};
use svg::{Document, Node};
use swash::FontRef;
use std::thread::available_parallelism;
use image::imageops;
use image::GenericImageView;
use image::imageops::grayscale;
use quadtree_rs::entry::Entry;
use crate::image::image::{average_color_for_rect, canny_algorithm, color_to_rgb_string};
use svg::node::element::Text;
use crate::io::debug::{debug_background_collision, debug_background_on_result, debug_collidables, debug_text};
use crate::types::rotation::Rotation::Ninety;


pub(crate) mod letter;
pub(crate) mod word;
mod word_cloud;

struct Inp {
    text: String,
    scale: f32,
    rotation: Rotation,
}

pub fn create_image() {
    let area_a = AreaBuilder::default()
        .anchor(
            (340, 991).into()
        )
        .dimensions(
            (5, 5)
        ).build().unwrap();
    let area_b = AreaBuilder::default()
        .anchor(
            (340, 991).into()
        )
        .dimensions(
            (7, 5)
        ).build().unwrap();


    assert!(Rect::from(&area_a).combine_rects(&Rect::from(&area_b)).is_some());

    let font_bts = include_bytes!("../../Lato-Regular.ttf") as &[u8];
    let test_image = include_bytes!("../../drake-nothing-was-the-same-148495.jpg") as &[u8];

    let random = SmallRng::from_entropy();
    let random_arc = Arc::new(Mutex::new(random));

    let mut inp: Vec<Inp> = Vec::new();
    let words = vec![
        "language",
        "building",
        "bad",
        "shade",
        "chop",
        "wiggly",
        "neighborly",
        "harm",
        "tacit",
        "anxious",
        "hushed",
        "tick",
        "bottle",
        "tank",
        "full",
        "sense",
        "push",
        "lumber",
        "damp",
        "yummy",
        "coal",
        "screw",
        "haunt",
        "itch",
        "linen",
        "ordinary",
        "noisy",
        "melt",
        "crowded",
        "parched",
        "bridge",
        "optimal",
        "easy",
        "suspect",
        "lackadaisical",
        "chicken",
        "basket",
        "pull",
        "glamorous",
        "invent",
        "small",
        "lick",
        "appreciate",
        "bake",
        "measure",
        "team",
        "lace",
        "dramatic",
        "knowledge",
        "elastic",
        "battle",
        "dispensable",
        "introduce",
        "x-ray",
        "dime",
        "accessible",
        "calm",
        "torpid",
        "encouraging",
        "wealthy",
        "careful",
        "silk",
        "efficacious",
        "suggestion",
        "ring",
        "beginner",
        "inquisitive",
        "bore",
        "messy",
        "side",
        "wreck",
        "tug",
        "sip",
        "faded",
        "vague",
        "ticket",
        "cattle",
        "outrageous",
        "shame",
        "allow",
        "imaginary",
        "burn",
        "arithmetic",
        "board",
        "suck",
        "attraction",
        "deserted",
        "gamy",
        "waste",
        "guitar",
        "worm",
        "nimble",
        "scribble",
        "substantial",
        "party",
        "hobbies",
        "petite",
        "ossified",
        "icicle",
        "believe",
        "hurried",
        "thrill",
        "abrupt",
        "grandiose",
        "whimsical",
        "pat",
        "analyze",
        "hideous",
        "moldy",
        "double",
        "bumpy",
        "cut",
        "repeat",
        "various",
        "legs",
        "fabulous",
        "ear",
        "harsh",
        "raise",
        "friends",
        "promise",
        "nod",
        "acidic",
        "silent",
        "doubtful",
        "paper",
        "wide",
        "squeal",
        "innocent",
        "spiteful",
        "woozy",
        "identify",
        "abrasive",
        "welcome",
        "windy",
        "listen",
        "rifle",
        "pizzas",
        "boundless",
        "moor",
        "airport",
        "detailed",
        "kittens",
        "jagged",
        "bleach",
        "scarf",
        "land",
        "hurt",
        "wealthy",
        "seashore",
        "muddled",
        "statuesque",
        "coat",
        "busy",
        "boundary",
        "scene",
        "umbrella",
        "dislike",
        "craven",
        "ragged",
        "impolite",
        "offer",
        "buzz",
        "spotted",
        "wriggle",
        "parallel",
        "night",
        "kiss",
        "pointless",
        "circle",
        "reproduce",
        "touch",
        "marble",
        "soap",
        "sip",
        "wall",
        "development",
        "arrogant",
        "charge",
        "tearful",
        "hurry",
        "calculate",
        "pine",
        "question",
        "glossy",
        "ruthless",
        "ethereal",
        "lamentable",
        "absorbing",
        "nauseating",
        "pig",
        "fuzzy",
        "tight",
        "nerve",
        "hole",
        "pushy",
        "doubt",
        "train",
        "parched",
        "warm",
    ];

    let mult = 1.0;

    let mut lock = random_arc.lock();

    for _ in 0..1 {
        inp.push(Inp {
            text: words[lock.gen_range(0..words.len())].parse().unwrap(),
            scale: lock.gen_range(80..100) as f32 * mult,
            rotation: Rotation::Zero,
        });
    }

    for _ in 0..0 {
        inp.push(Inp {
            text: words[lock.gen_range(0..words.len())].parse().unwrap(),
            scale: lock.gen_range(10..40) as f32 * mult,
            rotation: Rotation::Zero,
        });
    }

    for _ in 0..0 {
        inp.push(Inp {
            text: words[lock.gen_range(0..words.len())].parse().unwrap(),
            scale: lock.gen_range(5..20) as f32 * mult,
            rotation: Rotation::Zero,
        });
    }

    for _ in 0..0 {
        inp.push(Inp {
            text: words[lock.gen_range(0..words.len())].parse().unwrap(),
            scale: lock.gen_range(5..20) as f32 * mult,
            rotation: Rotation::Ninety,
        });
    }

    drop(lock);

    inp.sort_by(|x, y| y.scale.total_cmp(&x.scale));

    let document = Document::new()
        .set("viewBox", (0, 0, 1000, 1000))
        .set("height", 1000)
        .set("width", 1000);

    let width = 1000u32 * mult as u32;

    // Constants defining the spiral size.
    let a = 0.5_f64;
    let b = 5.0_f64;

    // max_angle = number of spirals * 2pi.
    let max_angle = 20.0_f64 * 2.0_f64 * std::f64::consts::PI;

    let quadtree_divisor: f32 = 4.;

    let depth = ((width as f32 / quadtree_divisor) as f64).log2() / 2.0_f64.log2();
    let mut quad_tree: Quadtree<u64, Word> = Quadtree::new(depth.ceil() as usize);


    const RESIZE_FACTOR: u32 = 1;

    let image = image::load_from_memory(test_image).expect("image load failed");
    let image_same_size_as_input = image.resize(width, width, imageops::FilterType::Nearest);
    let resized = image.resize(image.width() / RESIZE_FACTOR, image.height() / RESIZE_FACTOR, imageops::FilterType::Nearest);
    let greyed = grayscale(&resized);
    let detection = canny_algorithm(&greyed, 1.5);
    let detected_image = detection.as_image();

    let new_width = width / RESIZE_FACTOR;
    let depth = (new_width as f32 / quadtree_divisor).log2() / 2.0_f32.log2();
    let mut quadtree_boundaries: Quadtree<u64, u8> = Quadtree::new(depth.ceil() as usize);

    let mut layover = 0usize;
    let multiplier = width as f32 / detected_image.width() as f32;
    for (x, y, col) in detected_image.pixels() {
        if col.0[0] != 0 || col.0[1] != 0 || col.0[2] != 0 {
            let (pos_x, pos_y) = (
                f32::max(x as f32 * multiplier / quadtree_divisor - 1., 0.),
                f32::max(y as f32 * multiplier / quadtree_divisor - 1., 0.)
            );

            let search_area = AreaBuilder::default()
                .anchor(
                    (pos_x as u64, pos_y as u64).into()
                )
                .dimensions(
                    ((4. * multiplier / quadtree_divisor) as u64, (4. * multiplier / quadtree_divisor) as u64)
                ).build().unwrap();

            let insert_area = AreaBuilder::default()
                .anchor(
                    ((x as f32 * multiplier / quadtree_divisor) as u64, (y as f32 * multiplier / quadtree_divisor) as u64).into()
                )
                .dimensions(
                    ((1.) as u64, (1.) as u64)
                ).build().unwrap();

            let other = quadtree_boundaries.query(search_area).next();
            if let Some(o) = other {
                let comb = Rect::from(&insert_area).combine_rects(&Rect::from(&o.area()));
                if let Some(com) = comb {
                    let val = *o.value_ref();
                    quadtree_boundaries.delete_by_handle(o.handle());
                    quadtree_boundaries.insert(
                        Area::from(&com),
                        val + 1,
                    );
                    continue;
                } else {
                    layover += 1;
                }
            }
            quadtree_boundaries
                .insert(
                    insert_area, 0,
                );
        }
    }

    dbg!(
        quadtree_boundaries.len(), layover
    );

    let visible_space = Rect {
        min: Point { x: 0.0, y: 0.0 },
        max: Point {
            x: width as f32,
            y: width as f32,
        },
    };

    let font = FontRef::from_index(font_bts, 0).unwrap();

    let processing_slices = match available_parallelism() {
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
        .map(|inps|
            inps
                .iter()
                .map(|x|
                    {
                        let mut locked = random_arc.lock();
                        let left_offs = locked.gen_range(0.0..width as f32);
                        let top_offs = locked.gen_range(0.0..width as f32);
                        drop(locked);
                        Word::build_swash2(
                            &x.text,
                            font,
                            x.scale,
                            Point {
                                x: left_offs,
                                y: top_offs,
                            },
                            x.rotation,
                        )
                    }
                )
        )
        .flatten_iter()
        .collect::<Vec<Word>>();

    for mut word in words {
        let mut theta = 0.0_f64;
        let mut placed = false;
        let mut iters = 0;
        while theta < max_angle {
            if visible_space.contains(&word.bounding_box) {
                let mut intersected: bool = false;

                let search_region = AreaBuilder::default()
                    .anchor(quadtree_rs::point::Point {
                        x: f32::max((word.bounding_box.min.x / quadtree_divisor).ceil() - 1., 0.) as u64,
                        y: f32::max((word.bounding_box.min.y / quadtree_divisor).ceil() - 1., 0.) as u64,
                    })
                    .dimensions((
                        (word.bounding_box.width() / quadtree_divisor).ceil() as u64 + 2,
                        (word.bounding_box.height() / quadtree_divisor).ceil() as u64 + 2,
                    ))
                    .build()
                    .unwrap();

                let insert_region =
                    AreaBuilder::default()
                        .anchor(
                            (
                                (word.bounding_box.min.x / quadtree_divisor).ceil() as u64,
                                (word.bounding_box.min.y / quadtree_divisor).ceil() as u64
                            ).into()
                        )
                        .dimensions((
                            (word.bounding_box.width() / quadtree_divisor).ceil() as u64,
                            (word.bounding_box.height() / quadtree_divisor).ceil() as u64,
                        ))
                        .build()
                        .unwrap();

                let boundary_region = AreaBuilder::default()
                    .anchor(quadtree_rs::point::Point {
                        x: (word.bounding_box.min.x / RESIZE_FACTOR as f32 / quadtree_divisor).ceil() as u64,
                        y: (word.bounding_box.min.y / RESIZE_FACTOR as f32 / quadtree_divisor).ceil() as u64,
                    })
                    .dimensions((
                        (word.bounding_box.width() / RESIZE_FACTOR as f32 / quadtree_divisor).ceil() as u64,
                        (word.bounding_box.height() / RESIZE_FACTOR as f32 / quadtree_divisor).ceil() as u64,
                    ))
                    .build()
                    .unwrap();


                if quadtree_boundaries.query(boundary_region).next().is_some() {
                    intersected = true;
                }

                if !intersected {
                    for result in quad_tree.query(search_region) {
                        comparisons += 1;

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

                    match quad_tree.insert(insert_region, word) {
                        None => {
                            /*dbg!(new_region);
                            dbg!(quad_tree.height(), quad_tree.width());
                             */
                            panic!("insertion failed");
                        }
                        Some(_) => {}
                    }

                    break;
                }
            }

            if iters % 10 == 0 && iters > 0 {
                let mut lck = random_arc.lock();
                let new_pos = Point {
                    x: lck.gen_range(0.0..width as f32),
                    y: lck.gen_range(0.0..width as f32),
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

                let r = a + b * theta;

                let new_pos = Point {
                    x: ((r * theta.cos()) as i32 + word.offset.x as i32) as f32,
                    y: ((r * theta.sin()) as i32 + word.offset.y as i32) as f32,
                };

                iters += 1;

                word.move_word(&new_pos);
            }
            if iters % 25 == 0 {
                // dbg!("resizing");

                word = Word::build_swash2(
                    word.text.as_str(),
                    font,
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

    let entries: Vec<&Entry<u64, Word>> = quad_tree.iter().collect();
    let collected_entries: Vec<&Word> = quad_tree.iter().map(|x| x.value_ref()).collect();

    let sliced = collected_entries
        .as_slice()
        .splitn(
            (collected_entries.len() as f64 / processing_slices as f64).ceil() as usize,
            |_| false,
        )
        .collect::<Vec<&[&Word]>>();

    let doc_mutex = Arc::new(Mutex::new(document));

    sliced
        .par_iter()
        .for_each(
            |x| {
                for word in *x {
                    let integer_rect = Rect {
                        min: Point {
                            x: word.bounding_box.min.x as u32,
                            y: word.bounding_box.min.y as u32,
                        },
                        max: Point {
                            x: word.bounding_box.max.x as u32,
                            y: word.bounding_box.max.y as u32,
                        },
                    };
                    let avg_color = average_color_for_rect(&image_same_size_as_input, &integer_rect);
                    let p = Path::new()
                        .set("d", word.d())
                        .set("stoke", "none")
                        .set("fill", color_to_rgb_string(&avg_color));
                    let _s = p.to_string();
                    {
                        doc_mutex.lock().append(p);
                    }
                }
            }
        );
    svg::save("created.svg", &doc_mutex.lock().clone()).unwrap();

    if true {
        println!("Dumping Debug Files");
        if !std::path::Path::new("debug").is_dir() {
            create_dir("debug").unwrap();
        }
        debug_background_collision("debug/background_collision.svg", quadtree_boundaries.iter().collect::<Vec<&Entry<u64, u8>>>(), quadtree_divisor);
        debug_collidables("debug/collidables.svg", &entries);
        debug_text("debug/text.svg", &entries);
        debug_background_on_result("debug/text_on_background.svg", &entries, &quadtree_boundaries.iter().collect::<Vec<&Entry<u64, u8>>>(), quadtree_divisor);
    }
}
