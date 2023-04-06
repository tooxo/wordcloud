use crate::cloud::word::Inp;

use crate::cloud::word_cloud::create_word_cloud;
use crate::image::Dimensions;
use crate::types::rotation::Rotation;

use rand::{rngs::SmallRng, Rng, SeedableRng};

use swash::FontRef;

use crate::common::font::{Font, FontType};
use crate::rank::Word;

pub(crate) mod letter;
pub(crate) mod word;
mod word_cloud;

#[allow(dead_code)]
fn create_placeholder_words() -> Vec<Inp> {
    let mut random = SmallRng::from_seed([3; 32]);
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

    for _ in 0..30 {
        inp.push(Inp {
            text: words[random.gen_range(0..words.len())].parse().unwrap(),
            scale: random.gen_range(130..160) as f32,
            rotation: Rotation::Zero,
        });
    }

    for _ in 0..1000 {
        inp.push(Inp {
            text: words[random.gen_range(0..words.len())].parse().unwrap(),
            scale: random.gen_range(10..40) as f32,
            rotation: Rotation::Zero,
        });
    }

    for _ in 0..1000 {
        inp.push(Inp {
            text: words[random.gen_range(0..words.len())].parse().unwrap(),
            scale: random.gen_range(5..20) as f32,
            rotation: Rotation::Zero,
        });
    }

    for _ in 0..1000 {
        inp.push(Inp {
            text: words[random.gen_range(0..words.len())].parse().unwrap(),
            scale: random.gen_range(5..20) as f32,
            rotation: Rotation::Ninety,
        });
    }

    inp.sort_by(|x, y| y.scale.total_cmp(&x.scale));

    inp
}

pub fn create_image(input_words_counted: Vec<Word>) {
    let font_bts = include_bytes!("../../assets/OpenSans-Regular.ttf") as &[u8];
    let test_image = include_bytes!("../../assets/circ.png") as &[u8];

    let font_ref = FontRef::from_index(font_bts, 0).unwrap();
    let font = Font::new(font_ref, FontType::TTF);

    let image = image::load_from_memory(test_image).expect("image load failed");
    let output_dimensions = Dimensions::from_wh(1000, 1000);

    let words = 2000;
    let max = input_words_counted
        .iter()
        .take(words)
        .map(|x| (x.count as f32))
        .sum::<f32>()
        / words as f32;

    dbg!(max);

    let _inp: Vec<Inp> = input_words_counted
        .iter()
        .take(2000)
        .map(|w| Inp {
            text: w.content.to_string(),
            scale: ((w.count as f32) / max) * 15.,
            rotation: Rotation::Zero,
        })
        .collect();

    let inp = create_placeholder_words();
    /*let inp = vec![
        Inp{
            text: "aaaa".to_string(),
            scale: 90.0,
            rotation: Rotation::Ninety,
        }
    ];*/
    dbg!(inp.len());

    create_word_cloud(output_dimensions, font, inp, &image);
}
