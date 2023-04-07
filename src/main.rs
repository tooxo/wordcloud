use rayon::iter::ParallelIterator;
use rayon::prelude::ParallelString;
use std::path::Path;
use WordCloudRust::cloud::create_image;
use WordCloudRust::filtering::clean;
use WordCloudRust::filtering::stop_words::StopWords;
use WordCloudRust::io;
use WordCloudRust::io::folder::RecursiveFolderIterator;
use WordCloudRust::rank::RankedWords;

fn main() {
    let mut sw = StopWords::new().add_file("assets/stopwords");

    let folder_path = Path::new("./assets/stopwords-json").to_path_buf();
    let stopword_files = RecursiveFolderIterator::new(&folder_path, &|w| match w.file_name() {
        None => false,
        Some(n) => String::from(n.to_str().unwrap()).ends_with(".txt"),
    });

    for stopword_file in stopword_files {
        if let Some(fp) = stopword_file.to_str() {
            sw.append_file(fp);
        }
    }

    let s = io::file::read_string_from_file("input");
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
