use rayon::iter::ParallelIterator;
use rayon::prelude::ParallelString;
use WordCloudRust::cloud::create_image;
use WordCloudRust::filtering::clean;
use WordCloudRust::rank::RankedWords;
use WordCloudRust::{filtering, io};

fn main() {
    let sw = filtering::stop_words::StopWords::from_file("test");
    let s = io::file::read_string_from_file("lyric_test");
    let s2 = clean(s.to_lowercase().as_str());
    let f = s2
        .par_split_whitespace()
        .filter(|x| !sw.is_included(x))
        .filter(|x| x.parse::<f64>().is_err())
        .map(String::from)
        .collect::<Vec<String>>();

    let ranked = RankedWords::rank2(f);

    for i in ranked.iter().take(20) {
        println!("{}: {}x", i.content, i.count);
    }
    create_image(ranked);
}
