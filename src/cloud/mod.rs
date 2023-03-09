use std::ops::AddAssign;
use quadtree_rs::area::AreaBuilder;
use quadtree_rs::Quadtree;
use rand::{Rng, SeedableRng, thread_rng};
use rusttype::{Font, Scale};
use svg::{Document, Node};
use svg::node::element::{Group, Path, Rectangle};
use crate::types::point::Point;
use crate::types::rect::Rect;
use rand_chacha::ChaCha8Rng;
use svg::node::element::tag::Group;
use crate::cloud::word::Word;
use crate::types::rotation::Rotation;


mod word_cloud;
mod word;
mod letter;

struct Inp {
    text: String,
    scale: Scale,
    rotation: Rotation,
}

pub fn create_image() {
    let font_bts = include_bytes!("../../Lato-Regular.ttf") as &[u8];
    let font = Font::try_from_bytes(font_bts).unwrap();

    let mut random = ChaCha8Rng::seed_from_u64(12456576);
    // let mut random = thread_rng();

    let mut inp: Vec<Inp> = Vec::new();
    let words = vec!["language", "building", "bad", "shade", "chop", "wiggly", "neighborly", "harm", "tacit", "anxious", "hushed", "tick", "bottle", "tank", "full", "sense", "push", "lumber", "damp", "yummy", "coal", "screw", "haunt", "itch", "linen", "ordinary", "noisy", "melt", "crowded", "parched", "bridge", "optimal", "easy", "suspect", "lackadaisical", "chicken", "basket", "pull", "glamorous", "invent", "small", "lick", "appreciate", "bake", "measure", "team", "lace", "dramatic", "knowledge", "elastic", "battle", "dispensable", "introduce", "x-ray", "dime", "accessible", "calm", "torpid", "encouraging", "wealthy", "careful", "silk", "efficacious", "suggestion", "ring", "beginner", "inquisitive", "bore", "messy", "side", "wreck", "tug", "sip", "faded", "vague", "ticket", "cattle", "outrageous", "shame", "allow", "imaginary", "burn", "arithmetic", "board", "suck", "attraction", "deserted", "gamy", "waste", "guitar", "worm", "nimble", "scribble", "substantial", "party", "hobbies", "petite", "ossified", "icicle", "believe", "hurried", "thrill", "abrupt", "grandiose", "whimsical", "pat", "analyze", "hideous", "moldy", "double", "bumpy", "cut", "repeat", "various", "legs", "fabulous", "ear", "harsh", "raise", "friends", "promise", "nod", "acidic", "silent", "doubtful", "paper", "wide", "squeal", "innocent", "spiteful", "woozy", "identify", "abrasive", "welcome", "windy", "listen", "rifle", "pizzas", "boundless", "moor", "airport", "detailed", "kittens", "jagged", "bleach", "scarf", "land", "hurt", "wealthy", "seashore", "muddled", "statuesque", "coat", "busy", "boundary", "scene", "umbrella", "dislike", "craven", "ragged", "impolite", "offer", "buzz", "spotted", "wriggle", "parallel", "night", "kiss", "pointless", "circle", "reproduce", "touch", "marble", "soap", "sip", "wall", "development", "arrogant", "charge", "tearful", "hurry", "calculate", "pine", "question", "glossy", "ruthless", "ethereal", "lamentable", "absorbing", "nauseating", "pig", "fuzzy", "tight", "nerve", "hole", "pushy", "doubt", "train", "parched", "warm"];
    // let words = vec!["test", "tim"];

    let mult = 1.0;

    for _ in 0..1 {
        inp.push(
            Inp {
                text: words[random.gen_range(0..words.len())].parse().unwrap(),
                scale: Scale::uniform(random.gen_range(40..400) as f32 * mult),
                rotation: Rotation::Zero,
            }
        );
    }

    for _ in 0..000 {
        inp.push(
            Inp {
                text: words[random.gen_range(0..words.len())].parse().unwrap(),
                scale: Scale::uniform(random.gen_range(10..40) as f32 * mult),
                rotation: Rotation::Zero,
            }
        );
    }

    for _ in 0..000 {
        inp.push(
            Inp {
                text: words[random.gen_range(0..words.len())].parse().unwrap(),
                scale: Scale::uniform(random.gen_range(5..20) as f32 * mult),
                rotation: Rotation::Zero,
            }
        );
    }

    for _ in 0..1 {
        inp.push(
            Inp {
                text: words[random.gen_range(0..words.len())].parse().unwrap(),
                scale: Scale::uniform(random.gen_range(10..40) as f32 * mult),
                rotation: Rotation::Ninety,
            }
        );
    }

    inp.sort_by(
        |x, y| y.scale.x.total_cmp(&x.scale.x)
    );

    let mut document = Document::new()
        .set("viewBox", (0, 0, 1000, 1000))
        .set("height", 1000)
        .set("width", 1000);

    let width = 1000u32 * mult as u32;

    // Constants defining the spiral size.
    let a = 0.5_f64;
    let b = 5.0_f64;

    // max_angle = number of spirals * 2pi.
    let max_angle = 20.0_f64 * 2.0_f64 * std::f64::consts::PI;
    let width_quart = width / 8;

    let depth = (width as f64).log2() / 2.0_f64.log2();
    let mut quad_tree: Quadtree<u64, Word> = Quadtree::new(
        depth.ceil() as usize
    );

    let visible_space = Rect {
        min: Point { x: 0.0, y: 0.0 },
        max: Point { x: width as f32, y: width as f32 },
    };

    for word in inp {
        let mut theta = 0.0_f64;
        let left_offs = random.gen_range(width_quart..width_quart * 7);
        let top_offs = random.gen_range(width_quart..width_quart * 7);

        let mut w = Word::build(&word.text, &font, word.scale, Point { x: left_offs as f32, y: top_offs as f32 }, word.rotation);
        let mut placed = false;
        while theta < max_angle {
            if visible_space.contains(&w.bounding_box) {
                let mut intersected: bool = false;
                let path: Path = Path::new();

                let new_region = AreaBuilder::default()
                    .anchor(quadtree_rs::point::Point {
                        x: w.bounding_box.min.x as u64,
                        y: w.bounding_box.min.y as u64,
                    })
                    .dimensions(
                        (w.bounding_box.width() as u64, w.bounding_box.height() as u64)
                    )
                    .build()
                    .unwrap();

                let query = quad_tree.query(new_region);
                for result in query {
                    let other = result.value_ref();

                    let intersection = w.word_intersect(other);
                    if intersection.is_some() {
                        intersected = true;
                        break;
                    }
                }

                if !intersected {
                    println!("Placed {}", quad_tree.len());
                    placed = true;
                    if !path.get_attributes().is_empty() {
                        document.append(path);
                    }

                    quad_tree.insert(new_region, w);
                    break;
                }
            }

            let revelations = theta / (std::f64::consts::PI * 2.0f64);
            if revelations < 5.0 {
                theta += 1.0;
            } else {
                theta += 0.1_f64;
            }

            let r = a + b * theta;

            let new_pos = Point {
                x: ((r * theta.cos()) as i32 + left_offs as i32) as f32,
                y: ((r * theta.sin()) as i32 + top_offs as i32) as f32,
            };

            w.move_word(&new_pos);
        }

        if !placed {
            println!("Failed to place!");
        }
    }

    let mut document2 = Document::new()
        .set("viewBox", (0, 0, 1000, 1000))
        .set("height", 1000)
        .set("width", 1000);


    for entry in quad_tree.iter() {
        let path = entry.value_ref();
        let mut group = Group::new()
            .set("data-text", path.text.clone())
            .set("data-scale", path.scale.x)
            .set("fill", "black")
            .set("stroke", "none");
        for x in &path.glyphs {
            let p = Path::new()
                .set("d", x.d(&path.offset));
            group.append(
                p
            );
        }
        document.append(
            group
        );

        let rec = Rectangle::new()
            .set("x", path.bounding_box.min.x)
            .set("y", path.bounding_box.min.y)
            .set("width", path.bounding_box.width())
            .set("height", path.bounding_box.height())
            .set("stroke", "yellow")
            .set("stroke-width", 1)
            .set("fill", "none");

        document2.append(rec);
    }

    svg::save("test.svg", &document).unwrap();

    for x in quad_tree.iter() {
        let w = x.value_ref();
        for glyph in &w.glyphs {
            for x in glyph.absolute_collidables(&w.rotation, w.offset) {
                let p = Path::new()
                    .set("stroke", "black")
                    .set("stroke-width", 1)
                    .set("d",
                         format!(
                             "M {} {} L {} {} Z",
                             x.start.x, x.start.y, x.end.x, x.end.y
                         ),
                    );
                document2.append(p);
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

            document2.append(p);
        }
        /*for x in w.collidables() {
            let off = Point { x: 0.0, y: 0.0 };
            let line_s = x.to_string(&off);

            let p = Path::new()
                .set("stroke", "black")
                .set("stroke-width", 1)
                .set("d",
                     format!(
                         "M {} {} {} Z",
                         x.start.x, x.start.y, line_s,
                     ),
                );

            document2.append(p);
        }*/
    }

    svg::save("coll.svg", &document2).unwrap();
}