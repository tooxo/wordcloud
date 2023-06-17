use std::env::args;
use rayon::prelude::ParallelString;
use wordcloud::font::{Font, FontSetBuilder};
use wordcloud::{StopWordsIterator, StopWordsIteratorPar};
use wordcloud::{clean, Dimensions, WordCloudBuilder};
use wordcloud::{RankedWords, StopWords};
use rayon::iter::ParallelIterator;

fn main() {
    let sw = StopWords::default();

    let s = include_str!("assets/input");
    let s2 = clean(s.to_lowercase().as_str());

    let f = s2
        .par_split_whitespace()
        .filter_stop_words(&sw)
        .filter(|x| x.parse::<f64>().is_err())
        .map(String::from)
        .collect::<Vec<String>>();

    let ranked = RankedWords::rank(f);

    let mut font_bts = Vec::from(include_bytes!("assets/OpenSans-Regular.ttf") as &[u8]);
    let test_image = include_bytes!("assets/1000x1000bb.png") as &[u8];

    let font_set = FontSetBuilder::new()
        .push(Font::from_data(&mut font_bts).expect("couldn't parse font data"))
        .build();

    let image = image::load_from_memory(test_image).expect("image load failed");
    let output_dimensions = Dimensions::from_wh(1000, 1000);

    #[allow(clippy::unwrap_used)]
    let wc = WordCloudBuilder::new()
        .dimensions(output_dimensions)
        .font(&font_set)
        .image(&image)
        .build()
        .unwrap();

        wc.write_content(ranked, 2000);
    wc.export_text_to_file("created.svg")
        .expect("couldn't export");

    if args().any(|x| &x == "--debug") {
        println!("Dumping Debug Files");
        wc.export_debug_to_folder("debug/");
    }
}
