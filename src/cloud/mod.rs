use crate::cloud::word::Inp;

use crate::cloud::word_cloud::create_word_cloud;
use crate::image::image::Dimensions;
use crate::types::rotation::Rotation;
use parking_lot::Mutex;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::sync::Arc;
use swash::FontRef;

use crate::common::font::Font;

pub(crate) mod letter;
pub(crate) mod word;
mod word_cloud;

pub fn create_image() {
    let font_bts = include_bytes!("../../Lato-Regular.ttf") as &[u8];
    let test_image = include_bytes!("../../drake-nothing-was-the-same-148495.jpg") as &[u8];

    let font_ref = FontRef::from_index(font_bts, 0).unwrap();
    let font = Font::new(font_ref);

    let random = SmallRng::from_seed([3; 32]);
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

    for _ in 0..30 {
        inp.push(Inp {
            text: words[lock.gen_range(0..words.len())].parse().unwrap(),
            scale: lock.gen_range(130..160) as f32 * mult,
            rotation: Rotation::Zero,
        });
    }

    for _ in 0..2000 {
        inp.push(Inp {
            text: words[lock.gen_range(0..words.len())].parse().unwrap(),
            scale: lock.gen_range(10..40) as f32 * mult,
            rotation: Rotation::Zero,
        });
    }

    for _ in 0..2000 {
        inp.push(Inp {
            text: words[lock.gen_range(0..words.len())].parse().unwrap(),
            scale: lock.gen_range(5..20) as f32 * mult,
            rotation: Rotation::Zero,
        });
    }

    for _ in 0..2000 {
        inp.push(Inp {
            text: words[lock.gen_range(0..words.len())].parse().unwrap(),
            scale: lock.gen_range(5..20) as f32 * mult,
            rotation: Rotation::Ninety,
        });
    }
    drop(lock);

    inp.sort_by(|x, y| y.scale.total_cmp(&x.scale));

    let image = image::load_from_memory(test_image).expect("image load failed");
    let output_dimensions = Dimensions::from_wh(1000, 1000);

    create_word_cloud(output_dimensions, font, inp, &image);
}
