//! Select a segment handle before publishing an insertion.
use super::{
    checked_segment_component, hash_slice, Segment, SegmentedStorage, SegmentedStore, SetHandle,
};
use std::hash::Hash;

pub(crate) struct PreparedSegmentInsertion<'store, 'items, T: Clone, Id: Clone> {
    store: &'store mut SegmentedStore<T, Id>,
    items: &'items [T],
    id: Id,
    new_hash: Option<u64>,
}

impl<T: Clone + Hash + PartialEq, Id: SetHandle> SegmentedStore<T, Id> {
    /// Keeps the store exclusively borrowed so the selected handle cannot drift.
    /// Publication owns backing writes. Existing cold callers may rebuild the
    /// interner here; retained callers must first require prepared indexes.
    pub(crate) fn prepare_insertion<'store, 'items>(
        &'store mut self,
        items: &'items [T],
    ) -> PreparedSegmentInsertion<'store, 'items, T, Id> {
        if items.is_empty() {
            return PreparedSegmentInsertion {
                store: self,
                items,
                id: Id::EMPTY,
                new_hash: None,
            };
        }
        self.rebuild_interner_if_needed();
        let hash = hash_slice(items);
        if let Some(candidates) = self.interner.get(&hash) {
            for &candidate in candidates {
                if self.get(candidate) == items {
                    return PreparedSegmentInsertion {
                        store: self,
                        items,
                        id: candidate,
                        new_hash: None,
                    };
                }
            }
        }
        checked_segment_component(items.len(), "segment length");
        if let SegmentedStorage::Exclusive(flat) = &self.storage {
            checked_segment_component(flat.items.len(), "segment start");
        }
        let next = self
            .live_segment_count()
            .checked_add(1)
            .expect("segment handle overflow");
        checked_segment_component(next, "segment handle");
        PreparedSegmentInsertion {
            id: Id::from_index(next),
            store: self,
            items,
            new_hash: Some(hash),
        }
    }
}

impl<T: Clone + Hash + PartialEq, Id: SetHandle> PreparedSegmentInsertion<'_, '_, T, Id> {
    pub(crate) fn id(&self) -> Id {
        self.id
    }

    pub(crate) fn publish(self) -> Id {
        if let Some(hash) = self.new_hash {
            match &mut self.store.storage {
                SegmentedStorage::Exclusive(flat) => {
                    let start = checked_segment_component(flat.items.len(), "segment start");
                    flat.items.extend_from_slice(self.items);
                    flat.segments.push(Segment {
                        start,
                        len: checked_segment_component(self.items.len(), "segment length"),
                    });
                }
                SegmentedStorage::ForkShared { appended, .. } => {
                    appended.push_back(self.items.to_vec())
                }
            }
            self.store.interner.entry(hash).or_default().push(self.id);
        }
        self.id
    }
}
