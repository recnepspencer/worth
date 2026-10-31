//! Immutable native or decoded context storage; lookup meaning is unchanged.
use super::*;
type Entry = (
    WorthQueryCapabilityContextKey,
    worth_relational::facade::identity::EntityId,
);
pub(in crate::domain_computation::authorization) enum RetainedContext {
    Native(BTreeMap<WorthQueryCapabilityContextKey, worth_relational::facade::identity::EntityId>),
    Decoded(Vec<Entry>),
}
impl RetainedContext {
    pub(super) fn retained_storage_bytes(&self) -> Option<u64> {
        match self {
            Self::Native(entries) => crate::domain_computation::primary_graph::admitted_capture::storage::tree_clone_bytes::<Entry>(entries.len()),
            Self::Decoded(entries) => u64::try_from(entries.capacity().checked_mul(size_of::<Entry>())?).ok(),
        }
    }
    pub(super) fn decoded(mut entries: Vec<Entry>) -> Result<Self, ()> {
        entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(());
        }
        Ok(Self::Decoded(entries))
    }
    pub(in crate::domain_computation::authorization) fn iter(&self) -> ContextEntries<'_> {
        match self {
            Self::Native(entries) => ContextEntries::Native(entries.iter()),
            Self::Decoded(entries) => ContextEntries::Decoded(entries.iter()),
        }
    }
    pub(in crate::domain_computation::authorization) fn get(
        &self,
        key: &WorthQueryCapabilityContextKey,
    ) -> Option<&worth_relational::facade::identity::EntityId> {
        match self {
            Self::Native(entries) => entries.get(key),
            Self::Decoded(entries) => entries
                .binary_search_by(|entry| entry.0.cmp(key))
                .ok()
                .map(|index| &entries[index].1),
        }
    }
    pub(in crate::domain_computation::authorization) fn contains_key(
        &self,
        key: &WorthQueryCapabilityContextKey,
    ) -> bool {
        self.get(key).is_some()
    }
}
impl PartialEq for RetainedContext {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}
pub(in crate::domain_computation::authorization) enum ContextEntries<'a> {
    Native(
        std::collections::btree_map::Iter<
            'a,
            WorthQueryCapabilityContextKey,
            worth_relational::facade::identity::EntityId,
        >,
    ),
    Decoded(std::slice::Iter<'a, Entry>),
}
impl<'a> Iterator for ContextEntries<'a> {
    type Item = (
        &'a WorthQueryCapabilityContextKey,
        &'a worth_relational::facade::identity::EntityId,
    );
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Native(entries) => entries.next(),
            Self::Decoded(entries) => entries.next().map(|entry| (&entry.0, &entry.1)),
        }
    }
}
