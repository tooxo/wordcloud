use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct Word {
    content: String,
    count: usize,
}

impl Word {
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn count(&self) -> usize {
        self.count
    }
}

/**
    Used to precalculate the "common-ness" of the words.
*/
pub struct RankedWords(pub(crate) Vec<Word>);

impl RankedWords {
    /**
        Rank the words by accuracy.
    */
    pub fn rank(words: Vec<String>) -> RankedWords {
        let mut hs: HashMap<String, usize> = HashMap::new();
        for s in words {
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
