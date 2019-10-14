use crate::iter::BackwardIterableIndex;
use crate::IndexWithSA;

pub trait BackwardSearchIndex: BackwardIterableIndex {
    fn search_backward<K>(&self, pattern: K) -> Search<Self>
    where
        K: AsRef<[Self::T]>,
    {
        Search::new(self).search_backward(pattern)
    }
}

impl<I: BackwardIterableIndex> BackwardSearchIndex for I {}

pub struct Search<'a, I>
where
    I: BackwardIterableIndex,
{
    index: &'a I,
    s: u64,
    e: u64,
    pattern: Vec<I::T>,
}

impl<'a, I> Search<'a, I>
where
    I: BackwardIterableIndex,
{
    fn new(index: &'a I) -> Search<I> {
        Search {
            index,
            s: 0,
            e: index.len(),
            pattern: vec![],
        }
    }

    pub fn search_backward<K: AsRef<[I::T]>>(&self, pattern: K) -> Self {
        let mut s = self.s;
        let mut e = self.e;
        let mut pattern = pattern.as_ref().to_vec();
        for &c in pattern.iter().rev() {
            s = self.index.lf_map2(c, s);
            e = self.index.lf_map2(c, e);
            if s == e {
                break;
            }
        }
        pattern.extend_from_slice(&self.pattern);

        Search {
            index: self.index,
            s,
            e,
            pattern,
        }
    }

    pub fn get_range(&self) -> (u64, u64) {
        (self.s, self.e)
    }

    pub fn count(&self) -> u64 {
        self.e - self.s
    }
}

impl<'a, I> Search<'a, I>
where
    I: BackwardIterableIndex + IndexWithSA,
{
    pub fn locate(&self) -> Vec<u64> {
        let mut results: Vec<u64> = Vec::with_capacity((self.e - self.s) as usize);
        for k in self.s..self.e {
            results.push(self.index.get_sa(k));
        }
        results
    }
}
