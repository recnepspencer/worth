use super::{PersistentVector, PersistentVectorIter, PersistentVectorStorage};
use std::fmt;
use std::ops::{Index, IndexMut};

impl<T: Clone + fmt::Debug, const PAGE_LEN: usize> fmt::Debug for PersistentVector<T, PAGE_LEN> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Clone + PartialEq, const PAGE_LEN: usize> PartialEq for PersistentVector<T, PAGE_LEN> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<T: Clone + Eq, const PAGE_LEN: usize> Eq for PersistentVector<T, PAGE_LEN> {}

impl<T: Clone, const PAGE_LEN: usize> Default for PersistentVector<T, PAGE_LEN> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, const PAGE_LEN: usize> FromIterator<T> for PersistentVector<T, PAGE_LEN> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            storage: PersistentVectorStorage::Exclusive(iter.into_iter().collect()),
            retained_charge: None,
        }
    }
}

impl<T: Clone, const PAGE_LEN: usize> Index<usize> for PersistentVector<T, PAGE_LEN> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("persistent vector index in bounds")
    }
}

impl<T: Clone, const PAGE_LEN: usize> IndexMut<usize> for PersistentVector<T, PAGE_LEN> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
            .expect("persistent vector index in bounds")
    }
}

impl<'a, T: Clone, const PAGE_LEN: usize> IntoIterator for &'a PersistentVector<T, PAGE_LEN> {
    type Item = &'a T;
    type IntoIter = PersistentVectorIter<'a, T, PAGE_LEN>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
