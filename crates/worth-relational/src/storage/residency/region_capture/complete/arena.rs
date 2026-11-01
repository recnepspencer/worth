//! Borrowed SoA image fields; indexed column reads avoid iterator heap scratch.
use super::*;
use crate::storage::substrate::{
    AspectVersions, FieldRevisions, RecordArena, RecordKind, SharedColumn, SharedMap,
    StorageAllocation, StoredValue,
};
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};
use worth_foundational::facade::AuthoritativeRecordAspectState;

pub(in crate::storage::residency) trait WireKind: RecordKind {
    fn plan(
        schema: &RelationalBranchRootSchemaAuthority,
        kind: crate::identity::data::KindId,
    ) -> bool;
    fn meta_kind(meta: &Self::Meta) -> crate::identity::data::KindId;
    fn meta_state(meta: &Self::Meta) -> Option<&AuthoritativeRecordAspectState>;
    fn extra_state(extra: &Self::Extra) -> Option<&AuthoritativeRecordAspectState>;
    fn serialize_meta<S: serde::Serializer>(
        meta: &Self::Meta,
        serializer: S,
    ) -> Result<S::Ok, S::Error>;
    fn serialize_extra<S: serde::Serializer>(
        extra: &Self::Extra,
        serializer: S,
    ) -> Result<S::Ok, S::Error>;
}

pub(super) fn validate<K: WireKind>(
    arena: &RecordArena<K>,
    schema: &RelationalBranchRootSchemaAuthority,
    contracts: &aspects::Contracts<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Stop> {
    for physical in 0..arena.metadata_history.len() {
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        let rows = &arena.metadata_history[physical];
        for index in 0..rows.len() {
            request
                .consult()
                .map_err(|cause| Stop::Request(cause.into()))?;
            let meta = &rows[index];
            if !K::plan(schema, K::meta_kind(meta)) {
                return Err(invalid("original historical kind plan is absent"));
            }
            contracts.validate(K::meta_state(meta))?;
        }
    }
    for physical in 0..arena.extra.len() {
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        let state = K::extra_state(&arena.extra[physical]);
        if state.is_some() && !arena.kind_ids[physical].is_some_and(|kind| K::plan(schema, kind)) {
            return Err(invalid("original current aspect state has no kind plan"));
        }
        contracts.validate(state)?;
    }
    Ok(())
}

pub(super) struct Column<'a, T: StoredValue>(pub(super) &'a SharedColumn<T>);
impl<T: StoredValue> Serialize for Column<'_, T>
where
    T::Value: Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for index in 0..self.0.len() {
            sequence.serialize_element(&self.0[index])?;
        }
        sequence.end()
    }
}
struct ZeroPins(usize);
impl Serialize for ZeroPins {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0))?;
        for _ in 0..self.0 {
            sequence.serialize_element(&0u32)?;
        }
        sequence.end()
    }
}
struct Bitset<'a>(&'a crate::storage::partition::DenseSlotBitSet);
impl Serialize for Bitset<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct Words<'a>(&'a crate::storage::partition::DenseSlotBitSet);
        impl Serialize for Words<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut sequence = serializer.serialize_seq(Some(self.0.words().count()))?;
                for (index, word) in self.0.words() {
                    sequence.serialize_element(&(index as u64, word))?;
                }
                sequence.end()
            }
        }
        let mut fields = serializer.serialize_struct("DurableBitSet", 2)?;
        fields.serialize_field("words", &[] as &[u64])?;
        fields.serialize_field("sparse_words", &Words(self.0))?;
        fields.end()
    }
}

pub(in crate::storage::residency) struct MetadataSlice<'a, K: WireKind> {
    pub(in crate::storage::residency) rows: &'a SharedColumn<StorageAllocation<K::Meta>>,
    pub(in crate::storage::residency) range: std::ops::Range<usize>,
}
struct Metadata<'a, K: WireKind>(&'a K::Meta);
impl<K: WireKind> Serialize for Metadata<'_, K> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        K::serialize_meta(self.0, serializer)
    }
}
impl<K: WireKind> Serialize for MetadataSlice<'_, K> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.range.len()))?;
        for index in self.range.clone() {
            sequence.serialize_element(&Metadata::<K>(&self.rows[index]))?;
        }
        sequence.end()
    }
}
struct HeadMetadata<'a, K: WireKind>(&'a RecordArena<K>);
impl<K: WireKind> Serialize for HeadMetadata<'_, K> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.metadata_history.len()))?;
        for physical in 0..self.0.metadata_history.len() {
            let rows = &self.0.metadata_history[physical];
            sequence.serialize_element(&MetadataSlice::<K> {
                rows,
                range: rows.len().saturating_sub(1)..rows.len(),
            })?;
        }
        sequence.end()
    }
}
struct Extra<'a, K: WireKind>(&'a K::Extra);
impl<K: WireKind> Serialize for Extra<'_, K> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        K::serialize_extra(self.0, serializer)
    }
}
struct Extras<'a, K: WireKind>(&'a RecordArena<K>);
impl<K: WireKind> Serialize for Extras<'_, K> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.extra.len()))?;
        for physical in 0..self.0.extra.len() {
            sequence.serialize_element(&Extra::<K>(&self.0.extra[physical]))?;
        }
        sequence.end()
    }
}

pub(super) struct Arena<'a, K: WireKind>(pub(super) &'a RecordArena<K>);
impl<K: WireKind> Serialize for Arena<'_, K> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let arena = self.0;
        let mut fields = serializer.serialize_struct("RecordArenaCheckpointImage", 17)?;
        fields.serialize_field("slots", &Column(arena.slots.slots()))?;
        fields.serialize_field("generations", &Column(&arena.generations))?;
        fields.serialize_field("lifecycle", &Column(&arena.lifecycle))?;
        fields.serialize_field("kind_ids", &Column(&arena.kind_ids))?;
        fields.serialize_field("metadata_history", &HeadMetadata(arena))?;
        fields.serialize_field("created_at", &Column(&arena.created_at))?;
        fields.serialize_field("retired_at", &Column(&arena.retired_at))?;
        fields.serialize_field(
            "aspect_versions",
            &AspectVersionRows(&arena.aspect_versions),
        )?;
        fields.serialize_field(
            "field_revisions",
            &FieldRevisionRows(&arena.field_revisions),
        )?;
        fields.serialize_field("extra", &Extras(arena))?;
        fields.serialize_field(
            "diagnostics_enrichment",
            &Column(&arena.diagnostics_enrichment),
        )?;
        for name in ["branch_pins", "replay_pins", "snapshot_pins"] {
            fields.serialize_field(name, &ZeroPins(arena.generations.len()))?;
        }
        fields.serialize_field("live_bitset", &Bitset(&arena.live_bitset))?;
        fields.serialize_field("reclaimable_bitset", &Bitset(&arena.reclaimable_bitset))?;
        fields.serialize_field("free_list", &[] as &[u64])?;
        fields.end()
    }
}

// Borrow actual ordered nodes. Each map has a concrete known length, matching
// the prior BTreeMap grammar without staging an owned wire projection.
struct RevisionMap<'a, K: Copy + Ord, V: StoredValue>(&'a SharedMap<K, V>);
impl<K: Copy + Ord + Serialize, V: StoredValue> Serialize for RevisionMap<'_, K, V>
where
    V::Value: Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0.iter() {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}
struct AspectVersionRows<'a>(&'a SharedColumn<AspectVersions>);
impl Serialize for AspectVersionRows<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for physical in 0..self.0.len() {
            sequence.serialize_element(&RevisionMap(&self.0[physical]))?;
        }
        sequence.end()
    }
}
struct FieldRevisionRows<'a>(&'a SharedColumn<Option<FieldRevisions>>);
impl Serialize for FieldRevisionRows<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for physical in 0..self.0.len() {
            sequence.serialize_element(&self.0[physical].as_ref().map(RevisionMap))?;
        }
        sequence.end()
    }
}
