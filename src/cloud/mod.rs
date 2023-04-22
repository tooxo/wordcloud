use crate::cloud::word::Inp;

use crate::types::rotation::Rotation;
use rand::{rngs::SmallRng, Rng, SeedableRng};

pub(crate) mod letter;
pub(crate) mod word;
pub(crate) mod word_cloud;

pub use crate::cloud::word_cloud::{WordCloud, WordCloudBuilder};

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

    for _ in 0..20 {
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
