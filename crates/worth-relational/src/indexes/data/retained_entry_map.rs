//! Shared immutable entry backing for field and related-ordering generations.
use super::{DerivedIndexEntryMap, RelatedEntityOrderingEntry};
use crate::identity::data::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

// Prove thread safety at the index owner, where the concrete persistent
// tree is allocated. Generations carry that original backing as one share.
trait Backing<K: Ord + Clone, R: Clone>: Send + Sync {
    fn entries(&self) -> &DerivedIndexEntryMap<K, R>;
    fn entries_mut(&mut self) -> &mut DerivedIndexEntryMap<K, R>;
}

impl<K: Ord + Clone + Send + Sync, R: Clone + Send + Sync> Backing<K, R>
    for DerivedIndexEntryMap<K, R>
{
    fn entries(&self) -> &Self {
        self
    }

    fn entries_mut(&mut self) -> &mut Self {
        self
    }
}

#[derive(Clone)]
pub struct RetainedIndexEntryMap<K: Ord + Clone, R: Clone>(Arc<dyn Backing<K, R>>);

pub type RelatedEntityOrderingIndex = RetainedIndexEntryMap<EntityId, RelatedEntityOrderingEntry>;

impl<K: Ord + Clone, R: Clone> RetainedIndexEntryMap<K, R> {
    pub(crate) const fn header_bytes() -> u64 {
        (size_of::<DerivedIndexEntryMap<K, R>>() + 2 * size_of::<usize>()) as u64
    }

    pub(crate) fn shares_header(&self) -> bool {
        Arc::strong_count(&self.0) != 1
    }
}

impl<K: Ord + Clone + Send + Sync + 'static, R: Clone + Send + Sync + 'static>
    RetainedIndexEntryMap<K, R>
{
    /// The caller charges a shared header before detaching it.
    pub(crate) fn edit_map(&mut self) -> &mut DerivedIndexEntryMap<K, R> {
        if self.shares_header() {
            self.0 = Arc::new(self.0.entries().clone());
        }
        Arc::get_mut(&mut self.0)
            .expect("the index owner has exclusive header custody")
            .entries_mut()
    }
}

impl<K: Ord + Clone, R: Clone> Deref for RetainedIndexEntryMap<K, R> {
    type Target = DerivedIndexEntryMap<K, R>;

    fn deref(&self) -> &Self::Target {
        self.0.entries()
    }
}

#[cfg(test)]
impl<K: Ord + Clone + Send + Sync + 'static, R: Clone + Send + Sync + 'static> std::ops::DerefMut
    for RetainedIndexEntryMap<K, R>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.edit_map()
    }
}

impl<K: Ord + Clone + Send + Sync + 'static, R: Clone + Send + Sync + 'static> Default
    for RetainedIndexEntryMap<K, R>
{
    fn default() -> Self {
        DerivedIndexEntryMap::default().into()
    }
}

impl<K: Ord + Clone + Send + Sync + 'static, R: Clone + Send + Sync + 'static>
    From<DerivedIndexEntryMap<K, R>> for RetainedIndexEntryMap<K, R>
{
    fn from(entries: DerivedIndexEntryMap<K, R>) -> Self {
        Self(Arc::new(entries))
    }
}

impl<K: Ord + Clone + Send + Sync + 'static, R: Clone + Send + Sync + 'static>
    From<BTreeMap<K, Vec<R>>> for RetainedIndexEntryMap<K, R>
{
    fn from(entries: BTreeMap<K, Vec<R>>) -> Self {
        DerivedIndexEntryMap::from(entries).into()
    }
}

impl<K: Ord + Clone + fmt::Debug, R: Clone + fmt::Debug> fmt::Debug
    for RetainedIndexEntryMap<K, R>
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(formatter)
    }
}

impl<K: Ord + Clone, R: Clone + PartialEq> PartialEq for RetainedIndexEntryMap<K, R> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.deref() == other.deref()
    }
}

impl<K: Ord + Clone, R: Clone + Eq> Eq for RetainedIndexEntryMap<K, R> {}

impl<K: Ord + Clone + Serialize, R: Clone + Serialize> Serialize for RetainedIndexEntryMap<K, R> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.deref().serialize(serializer)
    }
}

impl<'de, K, R> Deserialize<'de> for RetainedIndexEntryMap<K, R>
where
    K: Ord + Clone + Send + Sync + Deserialize<'de> + 'static,
    R: Clone + Send + Sync + Deserialize<'de> + 'static,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        DerivedIndexEntryMap::deserialize(deserializer).map(Self::from)
    }
}
