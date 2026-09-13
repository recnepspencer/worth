use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use super::{checked_segment_component, FlatSegments, Segment, SegmentedStorage, SegmentedStore};

#[derive(Serialize, Deserialize)]
struct SegmentedStoreWire<T> {
    items: Vec<T>,
    segments: Vec<Segment>,
}

#[derive(Serialize)]
struct BorrowedSegmentedStoreWire<'a, T> {
    items: &'a [T],
    segments: &'a [Segment],
}

struct SharedItems<'a, T: Clone> {
    base: &'a FlatSegments<T>,
    appended: &'a crate::data::persistent_vector::PersistentVector<Vec<T>>,
}

impl<T: Clone + Serialize> Serialize for SharedItems<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let appended_len = self.appended.iter().map(Vec::len).sum::<usize>();
        let mut sequence = serializer.serialize_seq(Some(self.base.items.len() + appended_len))?;
        for value in &self.base.items {
            sequence.serialize_element(value)?;
        }
        for values in self.appended {
            for value in values {
                sequence.serialize_element(value)?;
            }
        }
        sequence.end()
    }
}

struct SharedSegments<'a, T: Clone> {
    base: &'a FlatSegments<T>,
    appended: &'a crate::data::persistent_vector::PersistentVector<Vec<T>>,
}

impl<T: Clone> Serialize for SharedSegments<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut sequence =
            serializer.serialize_seq(Some(self.base.segments.len() + self.appended.len()))?;
        for segment in &self.base.segments {
            sequence.serialize_element(segment)?;
        }
        let mut start = self.base.items.len();
        for values in self.appended {
            sequence.serialize_element(&Segment {
                start: checked_segment_component(start, "segment start"),
                len: checked_segment_component(values.len(), "segment length"),
            })?;
            start += values.len();
        }
        sequence.end()
    }
}

impl<T, Id> Serialize for SegmentedStore<T, Id>
where
    T: Clone + Serialize,
    Id: Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.storage {
            SegmentedStorage::Exclusive(flat) => BorrowedSegmentedStoreWire {
                items: &flat.items,
                segments: &flat.segments,
            }
            .serialize(serializer),
            SegmentedStorage::ForkShared { base, appended } => {
                let mut wire = serializer.serialize_struct("SegmentedStoreWire", 2)?;
                wire.serialize_field("items", &SharedItems { base, appended })?;
                wire.serialize_field("segments", &SharedSegments { base, appended })?;
                wire.end()
            }
        }
    }
}

impl<'de, T, Id> Deserialize<'de> for SegmentedStore<T, Id>
where
    T: Clone + Deserialize<'de>,
    Id: Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = SegmentedStoreWire::<T>::deserialize(deserializer)?;
        for segment in &wire.segments {
            let start = segment.start as usize;
            let end = start.saturating_add(segment.len as usize);
            wire.items.get(start..end).ok_or_else(|| {
                serde::de::Error::custom("segmented store range exceeds item storage")
            })?;
        }
        Ok(Self {
            storage: SegmentedStorage::Exclusive(FlatSegments {
                items: wire.items,
                segments: wire.segments,
            }),
            interner: crate::data::persistent_hash_map::PersistentHashMap::new(),
            id: PhantomData,
        })
    }
}
