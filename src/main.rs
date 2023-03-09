use WordCloudRust::{filtering, io};
use WordCloudRust::filtering::clean;
use WordCloudRust::rank::rank::RankedWords;
use rayon::iter::ParallelIterator;
use rayon::prelude::ParallelString;
use WordCloudRust::cloud::create_image;

fn main() {
    /*let sw = filtering::stop_words::StopWords::from_file("test");
    let s = io::file::read_string_from_file("lyric_test");
    let s2 = clean(s.to_lowercase().as_str());
    let f = s2.par_split_whitespace()
        .filter(|x| !sw.is_included(x))
        .filter(|x| x.parse::<f64>().is_err())
        .map(|x| String::from(x))
        .collect::<Vec<String>>();

    let ranked = RankedWords::rank2(f);

    for i in 0..200 {
        if ranked[i].count != 1 {
            println!("{}: {}x", ranked[i].content, ranked[i].count);
        }
    }*/

    create_image();
}