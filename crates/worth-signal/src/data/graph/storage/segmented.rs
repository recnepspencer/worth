mod fork_growth;
mod insertion;
mod persistent_fork;
use crate::data::retained_storage::RetainedStorageBacking;
use serde::{Deserialize, Serialize};

mod retained_charge;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

use super::handles::{DependencySetId, SetHandle, SubscriberSetId};
use crate::data::dependency::DependencyEdge;
use crate::data::handle::NodeId;

#[path = "segmented/equality.rs"]
mod equality;
#[cfg(test)]
#[path = "segmented/fork_granule_tests.rs"]
mod fork_granule_tests;
#[path = "segmented/serialization.rs"]
mod serialization;
#[cfg(test)]
#[path = "segmented/serialization_tests.rs"]
mod serialization_tests;
#[cfg(test)]
#[path = "segmented/value_tests.rs"]
mod value_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Segment {
    start: u32,
    len: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FlatSegments<T> {
    items: Vec<T>,
    segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
enum SegmentedStorage<T: Clone> {
    Exclusive(FlatSegments<T>),
    ForkShared {
        base: Arc<RetainedStorageBacking<FlatSegments<T>>>,
        appended: crate::data::persistent_vector::PersistentVector<Vec<T>>,
    },
}

#[derive(Debug)]
pub struct SegmentedStore<T: Clone, Id: Clone> {
    storage: SegmentedStorage<T>,
    interner: crate::data::persistent_hash_map::PersistentHashMap<u64, Vec<Id>>,
    id: PhantomData<Id>,
}

impl<T: Clone, Id: Clone> Clone for SegmentedStore<T, Id> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            interner: self.interner.clone(),
            id: PhantomData,
        }
    }
}

impl<T: Clone, Id: Clone> Default for SegmentedStore<T, Id> {
    fn default() -> Self {
        Self {
            storage: SegmentedStorage::Exclusive(FlatSegments {
                items: Vec::new(),
                segments: Vec::new(),
            }),
            interner: crate::data::persistent_hash_map::PersistentHashMap::new(),
            id: PhantomData,
        }
    }
}

impl<T, Id> SegmentedStore<T, Id>
where
    T: Clone + Hash + PartialEq,
    Id: SetHandle,
{
    fn rebuild_interner_if_needed(&mut self) {
        if !self.interner.is_empty() || self.live_segment_count() == 0 {
            return;
        }
        for index in 0..self.live_segment_count() {
            let segment = self.segment_at(index);
            self.interner
                .entry(hash_slice(segment))
                .or_default()
                .push(Id::from_index(index + 1));
        }
    }

    /// Constant-time guard against entering the lazy reconstruction branch.
    /// This is not a validation of arbitrary interner contents or provenance.
    pub(crate) fn retained_interner_requires_reconstruction(&self) -> bool {
        self.interner.is_empty() && self.live_segment_count() != 0
    }

    pub(crate) fn lookup_steps(&self) -> usize {
        match &self.storage {
            SegmentedStorage::Exclusive(_) => 4,
            SegmentedStorage::ForkShared { appended, .. } => appended.lookup_steps() + 4,
        }
    }

    pub fn get(&self, id: Id) -> &[T] {
        match id.index() {
            Some(index) => self.segment_at(index - 1),
            None => &[],
        }
    }

    pub fn insert_from_slice(&mut self, items: &[T]) -> Id {
        self.prepare_insertion(items).publish()
    }

    #[cfg(test)]
    pub(crate) fn storage_counts(&self) -> (usize, usize) {
        let item_count = match &self.storage {
            SegmentedStorage::Exclusive(flat) => flat.items.len(),
            SegmentedStorage::ForkShared { base, appended } => {
                base.items.len() + appended.iter().map(Vec::len).sum::<usize>()
            }
        };
        (item_count, self.live_segment_count())
    }

    pub(crate) fn live_segment_count(&self) -> usize {
        match &self.storage {
            SegmentedStorage::Exclusive(flat) => flat.segments.len(),
            SegmentedStorage::ForkShared { base, appended } => base.segments.len() + appended.len(),
        }
    }
}

impl<T, Id> SegmentedStore<T, Id>
where
    T: Clone,
    Id: Clone,
{
    pub(crate) fn operational_clone(&self) -> Self {
        let flat = match &self.storage {
            SegmentedStorage::Exclusive(flat) => flat.clone(),
            SegmentedStorage::ForkShared { base, appended } => {
                let appended_items = appended.iter().map(Vec::len).sum::<usize>();
                let mut flat = FlatSegments {
                    items: Vec::with_capacity(base.items.len() + appended_items),
                    segments: Vec::with_capacity(base.segments.len() + appended.len()),
                };
                flat.items.extend_from_slice(&base.items);
                flat.segments.extend_from_slice(&base.segments);
                for values in appended.iter() {
                    let start = checked_segment_component(flat.items.len(), "segment start");
                    flat.items.extend_from_slice(values);
                    flat.segments.push(Segment {
                        start,
                        len: checked_segment_component(values.len(), "segment length"),
                    });
                }
                flat
            }
        };
        Self {
            storage: SegmentedStorage::Exclusive(flat),
            interner: self.interner.operational_clone(),
            id: PhantomData,
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        match &self.storage {
            SegmentedStorage::Exclusive(_) => self.operational_clone(),
            SegmentedStorage::ForkShared { base, appended } => Self {
                storage: SegmentedStorage::ForkShared {
                    base: Arc::clone(base),
                    appended: appended.clone(),
                },
                interner: self.interner.fork_storage_identity(),
                id: PhantomData,
            },
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        matches!(
            (&self.storage, &other.storage),
            (
                SegmentedStorage::ForkShared { base: left, appended: left_tail },
                SegmentedStorage::ForkShared { base: right, appended: right_tail },
            ) if Arc::ptr_eq(left, right) && left_tail.shares_storage_with(right_tail)
        ) && self.interner.ptr_eq(&other.interner)
    }

    fn segment_at(&self, index: usize) -> &[T] {
        match &self.storage {
            SegmentedStorage::Exclusive(flat) => flat_slice(flat, index),
            SegmentedStorage::ForkShared { base, appended } => {
                if index < base.segments.len() {
                    flat_slice(base, index)
                } else {
                    appended[index - base.segments.len()].as_slice()
                }
            }
        }
    }
}

pub type DependencyEdgeStore = SegmentedStore<DependencyEdge, DependencySetId>;
pub type SubscriberEdgeStore = SegmentedStore<NodeId, SubscriberSetId>;

#[cfg(test)]
pub(crate) fn checked_segment_component_for_test(
    value: usize,
) -> Result<u32, crate::data::error::SignalError> {
    u32::try_from(value).map_err(|_| {
        crate::data::error::SignalError::invalid_input(format!(
            "edge-store segment component `{value}` exceeds u32 capacity"
        ))
    })
}

fn hash_slice<T: Hash>(items: &[T]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    items.hash(&mut hasher);
    hasher.finish()
}

fn flat_slice<T>(flat: &FlatSegments<T>, index: usize) -> &[T] {
    let segment = flat
        .segments
        .get(index)
        .expect("segmented store handle must index a live segment");
    let start = segment.start as usize;
    &flat.items[start..start + segment.len as usize]
}

fn checked_segment_component(value: usize, label: &str) -> u32 {
    u32::try_from(value).unwrap_or_else(|_| {
        panic!("worth-signal edge store overflow: {label} `{value}` exceeds u32 capacity")
    })
}
