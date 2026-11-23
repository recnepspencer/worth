//! Retained parent buckets for one related-entity ordering generation.
use super::{DerivedIndexEntryMap, RelatedEntityOrderingEntry};
use crate::identity::data::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

type Entries = DerivedIndexEntryMap<EntityId, RelatedEntityOrderingEntry>;

// Check the ordered trees' thread safety at the index owner before handing
// generations to callers. The concrete map still owns its persistent paths.
trait Backing: Send + Sync {
    fn entries(&self) -> &Entries;
    fn entries_mut(&mut self) -> &mut Entries;
}

impl Backing for Entries {
    fn entries(&self) -> &Entries {
        self
    }

    fn entries_mut(&mut self) -> &mut Entries {
        self
    }
}

#[derive(Clone)]
pub struct RelatedEntityOrderingIndex(Arc<dyn Backing>);

impl RelatedEntityOrderingIndex {
    pub(crate) const fn header_bytes() -> u64 {
        (size_of::<Entries>() + 2 * size_of::<usize>()) as u64
    }

    pub(crate) fn shares_header(&self) -> bool {
        Arc::strong_count(&self.0) != 1
    }

    /// The caller charges a shared header before detaching it.
    pub(crate) fn edit_map(&mut self) -> &mut Entries {
        if self.shares_header() {
            self.0 = Arc::new(self.0.entries().clone());
        }
        Arc::get_mut(&mut self.0)
            .expect("the index owner has exclusive header custody")
            .entries_mut()
    }
}

impl Deref for RelatedEntityOrderingIndex {
    type Target = Entries;

    fn deref(&self) -> &Entries {
        self.0.entries()
    }
}

#[cfg(test)]
impl std::ops::DerefMut for RelatedEntityOrderingIndex {
    fn deref_mut(&mut self) -> &mut Entries {
        self.edit_map()
    }
}

impl Default for RelatedEntityOrderingIndex {
    fn default() -> Self {
        Entries::default().into()
    }
}

impl From<Entries> for RelatedEntityOrderingIndex {
    fn from(entries: Entries) -> Self {
        Self(Arc::new(entries))
    }
}

impl From<BTreeMap<EntityId, Vec<RelatedEntityOrderingEntry>>> for RelatedEntityOrderingIndex {
    fn from(entries: BTreeMap<EntityId, Vec<RelatedEntityOrderingEntry>>) -> Self {
        Entries::from(entries).into()
    }
}

impl fmt::Debug for RelatedEntityOrderingIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.entries().fmt(formatter)
    }
}

impl PartialEq for RelatedEntityOrderingIndex {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0.entries() == other.0.entries()
    }
}

impl Eq for RelatedEntityOrderingIndex {}

impl Serialize for RelatedEntityOrderingIndex {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.entries().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RelatedEntityOrderingIndex {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Entries::deserialize(deserializer).map(Self::from)
    }
}
