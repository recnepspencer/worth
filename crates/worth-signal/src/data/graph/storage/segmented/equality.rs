use super::{FlatSegments, Segment, SegmentedStorage, SegmentedStore};

struct LogicalItems<'a, T: Clone> {
    base: std::slice::Iter<'a, T>,
    appended: Option<&'a crate::data::persistent_vector::PersistentVector<Vec<T>>>,
    appended_index: usize,
    item_index: usize,
}

impl<'a, T: Clone> LogicalItems<'a, T> {
    fn new(storage: &'a SegmentedStorage<T>) -> Self {
        match storage {
            SegmentedStorage::Exclusive(flat) => Self::from_parts(flat, None),
            SegmentedStorage::ForkShared { base, appended } => {
                Self::from_parts(base, Some(appended))
            }
        }
    }

    fn from_parts(
        flat: &'a FlatSegments<T>,
        appended: Option<&'a crate::data::persistent_vector::PersistentVector<Vec<T>>>,
    ) -> Self {
        Self {
            base: flat.items.iter(),
            appended,
            appended_index: 0,
            item_index: 0,
        }
    }
}

impl<'a, T: Clone> Iterator for LogicalItems<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.base.next() {
            return Some(item);
        }
        let appended = self.appended?;
        while self.appended_index < appended.len() {
            let values = &appended[self.appended_index];
            if let Some(item) = values.get(self.item_index) {
                self.item_index += 1;
                return Some(item);
            }
            self.appended_index += 1;
            self.item_index = 0;
        }
        None
    }
}

struct LogicalSegments<'a, T: Clone> {
    base: std::slice::Iter<'a, Segment>,
    appended: Option<&'a crate::data::persistent_vector::PersistentVector<Vec<T>>>,
    appended_index: usize,
    next_start: usize,
}

impl<'a, T: Clone> LogicalSegments<'a, T> {
    fn new(storage: &'a SegmentedStorage<T>) -> Self {
        match storage {
            SegmentedStorage::Exclusive(flat) => Self::from_parts(flat, None),
            SegmentedStorage::ForkShared { base, appended } => {
                Self::from_parts(base, Some(appended))
            }
        }
    }

    fn from_parts(
        flat: &'a FlatSegments<T>,
        appended: Option<&'a crate::data::persistent_vector::PersistentVector<Vec<T>>>,
    ) -> Self {
        Self {
            base: flat.segments.iter(),
            appended,
            appended_index: 0,
            next_start: flat.items.len(),
        }
    }
}

impl<T: Clone> Iterator for LogicalSegments<'_, T> {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(segment) = self.base.next() {
            return Some(*segment);
        }
        let appended = self.appended?;
        let values = appended.get(self.appended_index)?;
        let segment = Segment {
            start: super::checked_segment_component(self.next_start, "segment start"),
            len: super::checked_segment_component(values.len(), "segment length"),
        };
        self.appended_index += 1;
        self.next_start += values.len();
        Some(segment)
    }
}

impl<T, Id> PartialEq for SegmentedStore<T, Id>
where
    T: Clone + PartialEq,
    Id: Clone + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.interner != other.interner {
            return false;
        }
        if let (SegmentedStorage::Exclusive(left), SegmentedStorage::Exclusive(right)) =
            (&self.storage, &other.storage)
        {
            return left == right;
        }
        LogicalItems::new(&self.storage).eq(LogicalItems::new(&other.storage))
            && LogicalSegments::new(&self.storage).eq(LogicalSegments::new(&other.storage))
    }
}

impl<T, Id> Eq for SegmentedStore<T, Id>
where
    T: Clone + Eq,
    Id: Clone + Eq,
{
}
