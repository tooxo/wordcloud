use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::collections::{HashMap};

#[derive(Clone)]
pub struct Word {
    pub content: String,
    pub count: usize,
}

impl Word {
    pub fn new(content: String) -> Self {
        Self { content, count: 1 }
    }
}

pub struct RankedWords(pub Vec<Word>);

impl RankedWords {
    pub fn rank(v: Vec<String>) -> RankedWords {
        let mut hs: HashMap<String, usize> = HashMap::new();
        for s in v {
            let f = hs.get(&s);
            hs.insert(s, *f.unwrap_or(&0) + 1);
        }

        let mut n = hs
            .into_par_iter()
            .map(|c| (c.0, c.1))
            .map(|k| Word {
                content: k.0,
                count: k.1,
            })
            .collect::<Vec<Word>>();

        n.sort_by(|w, w2| w2.count.cmp(&w.count));

        RankedWords(n)
    }
}
