//! Borrowed-or-owned row handle for [`super::TieredMap`] reads that does not
//! require `V: Clone` (so `!Clone` state owners like `NameEntry` stay fenced).
use std::ops::Deref;

/// A row value either borrowed from the resident delta or decoded from a cold page.
#[derive(Debug)]
pub enum RowRef<'a, V> {
    Borrowed(&'a V),
    Owned(V),
}

impl<V: PartialEq> PartialEq for RowRef<'_, V> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<V: Eq> Eq for RowRef<'_, V> {}

impl<V> Deref for RowRef<'_, V> {
    type Target = V;

    fn deref(&self) -> &V {
        match self {
            Self::Borrowed(value) => value,
            Self::Owned(value) => value,
        }
    }
}

impl<V> RowRef<'_, V> {
    pub fn into_owned_with(self, duplicate: impl FnOnce(&V) -> V) -> V {
        match self {
            Self::Borrowed(value) => duplicate(value),
            Self::Owned(value) => value,
        }
    }
}

impl<V: Clone> RowRef<'_, V> {
    pub fn into_owned(self) -> V {
        self.into_owned_with(Clone::clone)
    }
}
