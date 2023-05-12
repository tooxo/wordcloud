use crate::StopWords;
use rayon::iter::plumbing::{Consumer, Folder, UnindexedConsumer};
use rayon::iter::ParallelIterator;

pub struct StopWordsIteratorInner<'a, I>
where
    I: Iterator,
{
    sw: &'a StopWords,
    underlying: I,
}

pub struct StopWordsIteratorParallel<'a, I>
where
    I: ParallelIterator,
{
    base: I,
    sw: &'a StopWords,
}

impl<'a, I> Iterator for StopWordsIteratorInner<'a, I>
where
    I: Iterator,
    I::Item: ToString,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.underlying
            .by_ref()
            .find(|next| !self.sw.is_included(&next.to_string()))
    }
}

impl<'a, I> ParallelIterator for StopWordsIteratorParallel<'a, I>
where
    I: ParallelIterator,
    I::Item: ToString,
{
    type Item = I::Item;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let consumer1 = StopWordConsumer {
            base: consumer,
            sw: self.sw,
        };
        self.base.drive_unindexed(consumer1)
    }
}

struct StopWordConsumer<'a, C> {
    base: C,
    sw: &'a StopWords,
}

impl<'a, T, C> Consumer<T> for StopWordConsumer<'a, C>
where
    C: Consumer<T>,
    T: ToString,
{
    type Folder = StopWordFilterFolder<'a, C::Folder>;
    type Reducer = C::Reducer;
    type Result = C::Result;

    fn split_at(self, index: usize) -> (Self, Self, Self::Reducer) {
        let (left, right, reducer) = self.base.split_at(index);
        (
            StopWordConsumer {
                base: left,
                sw: self.sw,
            },
            StopWordConsumer {
                base: right,
                sw: self.sw,
            },
            reducer,
        )
    }

    fn into_folder(self) -> Self::Folder {
        StopWordFilterFolder {
            base: self.base.into_folder(),
            sw: self.sw,
        }
    }

    fn full(&self) -> bool {
        self.base.full()
    }
}

impl<'a, T, C> UnindexedConsumer<T> for StopWordConsumer<'a, C>
where
    C: UnindexedConsumer<T>,
    T: ToString,
{
    fn split_off_left(&self) -> Self {
        StopWordConsumer {
            base: self.base.split_off_left(),
            sw: self.sw,
        }
    }

    fn to_reducer(&self) -> Self::Reducer {
        self.base.to_reducer()
    }
}

struct StopWordFilterFolder<'a, C> {
    base: C,
    sw: &'a StopWords,
}

impl<'a, C, T> Folder<T> for StopWordFilterFolder<'a, C>
where
    C: Folder<T>,
    T: ToString,
{
    type Result = C::Result;

    fn consume(self, item: T) -> Self {
        if !self.sw.is_included(&item.to_string()) {
            let base = self.base.consume(item);
            StopWordFilterFolder { base, sw: self.sw }
        } else {
            self
        }
    }

    fn complete(self) -> Self::Result {
        self.base.complete()
    }

    fn full(&self) -> bool {
        self.base.full()
    }
}

/**
    Trait implementing an [`Iterator`] function to filter out StopWords out of an Iterator
*/
pub trait StopWordsIterator: Iterator {
    fn filter_stop_words(self, sw: &StopWords) -> StopWordsIteratorInner<Self>
    where
        Self: Sized,
    {
        StopWordsIteratorInner {
            sw,
            underlying: self,
        }
    }
}

/**
   Trait implementing a [`ParallelIterator`] function to filter out StopWords out of a ParallelIterator
*/
pub trait StopWordsIteratorPar: ParallelIterator {
    fn filter_stop_words(self, sw: &StopWords) -> StopWordsIteratorParallel<Self>
    where
        Self: Sized,
    {
        StopWordsIteratorParallel { sw, base: self }
    }
}

impl<I: Iterator> StopWordsIterator for I {}
impl<P: ParallelIterator> StopWordsIteratorPar for P {}

#[test]
fn test_stop_word_filter() {
    let mut sw = StopWords::new();
    sw.append_words(vec!["StopWord"]);

    let c: Vec<_> = vec!["a", "StopWord", "d", "e"]
        .into_iter()
        .map(String::from)
        .collect();

    let filtered: Vec<_> = c.into_iter().filter_stop_words(&sw).collect();
    assert!(filtered.contains(&"a".to_string()));
    assert!(filtered.contains(&"d".to_string()));
    assert!(filtered.contains(&"e".to_string()));
    assert!(!filtered.contains(&"StopWord".to_string()));
}

#[test]
fn test_stop_word_filter_parallel() {
    use rayon::iter::IntoParallelIterator;
    use rayon::iter::ParallelIterator;

    let mut sw = StopWords::new();
    sw.append_words(vec!["StopWord"]);

    let c: Vec<_> = vec!["a", "StopWord", "d", "e"]
        .into_iter()
        .map(String::from)
        .collect();

    let filtered: Vec<_> = c.into_par_iter().filter_stop_words(&sw).collect();

    assert!(filtered.contains(&"a".to_string()));
    assert!(filtered.contains(&"d".to_string()));
    assert!(filtered.contains(&"e".to_string()));
    assert!(!filtered.contains(&"StopWord".to_string()));
}
