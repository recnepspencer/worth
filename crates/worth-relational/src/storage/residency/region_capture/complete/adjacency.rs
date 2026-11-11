//! Borrow the same sorted adjacency/kind/revision values as ordinary images.
use super::*;
use crate::identity::data::KindId;
use crate::storage::partition::{AdjacencyKindBasis, AdjacencySet, SparseAdjacencyTable};
use serde::ser::{SerializeSeq, SerializeStruct};
struct Ids<'a>(&'a AdjacencySet);
impl Serialize for Ids<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.current_ids().iter())
    }
}
struct Revisions<'a>(&'a AdjacencySet);
impl Serialize for Revisions<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.structural_revision_count()))?;
        for revision in self.0.structural_revision_entries() {
            sequence.serialize_element(&revision)?;
        }
        sequence.end()
    }
}
pub(super) struct Table<'a>(pub(super) &'a SparseAdjacencyTable);
impl Serialize for Table<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Entry<'a> {
            slot: u64,
            relations: Ids<'a>,
            structural_revisions: Revisions<'a>,
        }
        serializer.collect_seq(self.0.iter().map(|(&slot, set)| Entry {
            slot: slot as u64,
            relations: Ids(set),
            structural_revisions: Revisions(set),
        }))
    }
}
pub(super) struct Kinds<'a>(
    pub(super) &'a SparseAdjacencyTable,
    pub(super) AdjacencyKindBasis,
);
struct KindRows<'a>(&'a AdjacencySet, AdjacencyKindBasis);
struct KindIds<'a>(&'a AdjacencySet, AdjacencyKindBasis, KindId);
impl Serialize for KindIds<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let ids = match self.1 {
            AdjacencyKindBasis::Current => self.0.current_kind_ids(self.2),
            AdjacencyKindBasis::Historical => self.0.historical_kind_ids(self.2),
        };
        serializer.collect_seq(ids.iter())
    }
}
impl Serialize for KindRows<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut prior = None;
        let mut count = 0;
        for (kind, _) in self.0.kind_memberships(self.1) {
            if prior != Some(kind) {
                count += 1;
                prior = Some(kind);
            }
        }
        let mut sequence = serializer.serialize_seq(Some(count))?;
        let mut prior = None;
        for (kind, _) in self.0.kind_memberships(self.1) {
            if prior == Some(kind) {
                continue;
            }
            prior = Some(kind);
            sequence.serialize_element(&(kind, KindIds(self.0, self.1, kind)))?;
        }
        sequence.end()
    }
}
impl Serialize for Kinds<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct Entry<'a> {
            slot: u64,
            kinds: KindRows<'a>,
        }
        impl Serialize for Entry<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut fields = serializer.serialize_struct("RegionAdjacencyKinds", 2)?;
                fields.serialize_field("slot", &self.slot)?;
                fields.serialize_field("kinds", &self.kinds)?;
                fields.end()
            }
        }
        serializer.collect_seq(self.0.iter().map(|(&slot, set)| Entry {
            slot: slot as u64,
            kinds: KindRows(set, self.1),
        }))
    }
}
