use rayon::iter::ParallelIterator;
use rayon::str::ParallelString;
use wordcloud::common::{Font, FontSetBuilder};
use wordcloud::io::file;
use wordcloud::rank::RankedWords;
use wordcloud::StopWords;
use wordcloud::{clean, Dimensions, WordCloudBuilder};

pub fn create_image(input_words_counted: RankedWords) {
    let mut font_bts = Vec::from(include_bytes!("../assets/OpenSans-Regular.ttf") as &[u8]);
    let test_image = include_bytes!("../drake-nothing-was-the-same-148495.jpg") as &[u8];

    let font_set = FontSetBuilder::new()
        .push(Font::from_data(&mut font_bts))
        .build();

    let image = image::load_from_memory(test_image).expect("image load failed");
    let output_dimensions = Dimensions::from_wh(1000, 1000);

    let wc = WordCloudBuilder::new()
        .dimensions(output_dimensions)
        .font(&font_set)
        .image(&image)
        .build()
        .unwrap();

    wc.write_content(input_words_counted, 2000);
    wc.write_to_file("created.svg");

    if true {
        println!("Dumping Debug Files");

        wc.write_debug_to_folder("debug/");
    }
}

fn main() {
    let sw = StopWords::default();

    let s = file::read_string_from_file("input");
    let s2 = clean(s.to_lowercase().as_str());
    let f = s2
        .par_split_whitespace()
        .filter(|x| !sw.is_included(x))
        .filter(|x| x.parse::<f64>().is_err())
        .map(String::from)
        .collect::<Vec<String>>();

    let ranked = RankedWords::rank(f);

    for i in ranked.0.iter().take(20) {
        println!("{}: {}x", i.content, i.count);
    }
    create_image(ranked);
}
