use rayon::iter::ParallelIterator;
use rayon::str::ParallelString;
use wordcloud::clean;
use wordcloud::create_image;
use wordcloud::io::file;
use wordcloud::rank::RankedWords;
use wordcloud::StopWords;

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

    let ranked = RankedWords::rank2(f);

    for i in ranked.iter().take(20) {
        println!("{}: {}x", i.content, i.count);
    }
    create_image(ranked);
}
