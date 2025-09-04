use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderedVec<T>
where
    T: std::cmp::Ord,
{
    vec: Vec<T>,
}

impl<T> OrderedVec<T>
where
    T: std::cmp::Ord,
{
    #[allow(dead_code)]
    pub fn insert(&mut self, value: T) {
        match self.vec.binary_search(&value) {
            Ok(_) => {}
            Err(pos) => self.vec.insert(pos, value),
        }
    }
    #[allow(dead_code)]
    pub fn last(&self) -> Option<&T> {
        self.vec.last()
    }
    #[allow(dead_code)]
    pub fn get<Idx>(&self, index: Idx) -> Option<&T>
    where
        Idx: std::slice::SliceIndex<[T], Output = T>,
    {
        self.vec.get(index)
    }
    pub fn len(&self) -> usize {
        self.vec.len()
    }
    pub fn get_slice(&self, range: std::ops::Range<usize>) -> &[T] {
        &self.vec[range]
    }
}

impl<T, Idx> std::ops::Index<Idx> for OrderedVec<T>
where
    Idx: std::slice::SliceIndex<[T], Output = T>,
    T: std::cmp::Ord,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.vec[index]
    }
}

impl<T> IntoIterator for OrderedVec<T>
where
    T: Ord,
{
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}
