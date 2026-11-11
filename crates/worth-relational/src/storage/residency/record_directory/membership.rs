//! One record domain's kind-prefixed membership and key-shaped commitment.
use crate::identity::data::{KindId, PartitionId, RecordId};
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::storage::substrate::{DigestIndex, SharedMap};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default)]
pub(super) struct Membership {
    records: SharedMap<u128, ()>,
    digest: DigestIndex,
}
impl Membership {
    pub(super) fn digest(&self) -> [u8; 32] {
        self.digest.digest()
    }
    pub(super) fn keys(&self) -> impl Iterator<Item = u128> + '_ {
        self.records.keys().copied()
    }
    pub(super) fn changed_kinds(&self, previous: &Self) -> BTreeSet<KindId> {
        self.records
            .changed_keys_since(&previous.records)
            .into_iter()
            .map(kind)
            .collect()
    }
    pub(super) fn ids<K: Copy>(
        &self,
        selected: KindId,
        partition: PartitionId,
    ) -> impl Iterator<Item = RecordId<K>> + '_ {
        self.records
            .iter_from(&((selected.0 as u128) << 96))
            .take_while(move |(key, _)| kind(**key) == selected)
            .map(move |(key, _)| identity(*key, partition))
    }
    pub(super) fn cold(
        lane: &ReconstructionLane<'_>,
        entries: Vec<u128>,
    ) -> Result<Self, super::maintenance::DirectoryError> {
        let digests: Vec<_> = entries.iter().map(|key| (*key, [1; 32])).collect();
        let digest = DigestIndex::cold(lane, &digests)
            .map_err(|_| super::maintenance::DirectoryError::Corrupt)?;
        Ok(Self {
            records: SharedMap::from_sorted_unique(
                entries.into_iter().map(|key| (key, ())).collect(),
            ),
            digest,
        })
    }
    pub(super) fn update(
        &mut self,
        key: u128,
        present: bool,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), super::maintenance::DirectoryError>,
    ) -> Result<(), super::maintenance::DirectoryError> {
        if self.records.contains_key(&key) == present {
            return Ok(());
        }
        let (units, digest_bytes) = self.digest.edit_quote(key);
        let bytes = self
            .records
            .adoption_allocation_bound()
            .and_then(|bytes| bytes.checked_add(digest_bytes))
            .ok_or(super::maintenance::DirectoryError::Corrupt)?;
        prepare(units + self.records.navigation_work_bound() as u64, bytes)?;
        if present {
            self.records.insert(key, ());
        } else {
            self.records.remove(&key);
        }
        self.digest
            .set(key, present.then_some([1; 32]))
            .map_err(|_| super::maintenance::DirectoryError::Corrupt)
    }
}
pub(super) fn key<K: Copy>(kind: KindId, record: RecordId<K>) -> u128 {
    ((kind.0 as u128) << 96)
        | ((record.local_slot_value() as u128) << 32)
        | record.generation_value() as u128
}
pub(super) fn kind(key: u128) -> KindId {
    KindId((key >> 96) as u32)
}
pub(super) fn identity<K>(key: u128, partition: PartitionId) -> RecordId<K> {
    RecordId::new(partition, (key >> 32) as u64, key as u32)
}
