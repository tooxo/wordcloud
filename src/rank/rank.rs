use rayon::iter::IntoParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::collections::{HashMap, HashSet};

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

pub struct RankedWords {}

impl RankedWords {
    pub fn rank(v: Vec<String>) {
        let v_v = v.clone();
        let mut n = v
            .into_iter()
            .collect::<HashSet<String>>()
            .par_iter()
            .map(|val| Word {
                content: val.clone(),
                count: v_v
                    .par_iter()
                    .map(|v2| val.eq(v2))
                    .filter(|x| *x)
                    .collect::<Vec<bool>>()
                    .len(),
            })
            .collect::<Vec<Word>>();

        n.sort_by(|w, w2| w.count.cmp(&w2.count));

        println!("-");
    }

    pub fn rank2(v: Vec<String>) -> Vec<Word> {
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

        n
    }
}
