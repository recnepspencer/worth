mod reconstruction;
use super::{hash_value, PartitionContentDigestError};
use super::{ReconstructionLane, ReconstructionWork};
use crate::storage::substrate::DigestIndex;
use crate::storage::substrate::{
    HistoryAppendFrontier, RecordArena, RecordKind, SharedMap, StorageAllocationVisitor,
};
use crate::symbols::data::{StringInterner, Symbol};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub(super) struct RecordContentIndex {
    records: DigestIndex,
    histories: SharedMap<u64, DigestIndex>,
    history_bytes: u64,
    live_words: DigestIndex,
    reclaimable_words: DigestIndex,
}

impl RecordContentIndex {
    pub(super) fn history_frontiers<K: RecordKind>(
        &self,
        _lane: &ReconstructionLane<'_>,
        arena: &RecordArena<K>,
        work: &mut ReconstructionWork<'_, '_>,
    ) -> Result<Vec<(u64, HistoryAppendFrontier)>, PartitionContentDigestError>
    where
        K::Meta: serde::Serialize,
    {
        work.prepare(
            0,
            (self.histories.len() * size_of::<(u64, HistoryAppendFrontier)>()) as u64,
        )?;
        let mut result = Vec::with_capacity(self.histories.len());
        for (&slot, history) in self.histories.iter() {
            let (units, bytes) = history
                .frontier_quote()
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
            work.prepare(units, bytes)?;
            let metadata = arena
                .metadata_history_at(
                    usize::try_from(slot)
                        .map_err(|_| PartitionContentDigestError::InvalidHistoryCoverage)?,
                )
                .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
            let last_value = metadata
                .len()
                .checked_sub(1)
                .and_then(|index| metadata.get(index))
                .map(|value| super::admission::hash_controlled(b"metadata", value, work))
                .transpose()?;
            result.push((
                slot,
                history
                    .append_frontier(last_value)
                    .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?,
            ));
        }
        Ok(result)
    }

    pub(super) fn update<K: RecordKind>(
        &mut self,
        arena: &RecordArena<K>,
        previous: &RecordArena<K>,
        slots: impl IntoIterator<Item = usize>,
        symbols: &StringInterner,
        extra: impl Fn(&K::Extra, &StringInterner) -> Result<[u8; 32], PartitionContentDigestError>,
    ) -> Result<u64, PartitionContentDigestError>
    where
        K::Meta: serde::Serialize,
    {
        let mut hashed_values = 0;
        for slot in slots {
            let physical = arena
                .physical_index(slot)
                .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
            let cached = self
                .histories
                .get(&(slot as u64))
                .cloned()
                .unwrap_or_default();
            let old_bytes = cached.allocation_bytes();
            let (history, hashed) = super::record_history::update(arena, previous, slot, &cached)?;
            hashed_values += hashed;
            let versions = resolve_aspect_versions(&arena.aspect_versions[physical], symbols)?;
            let digest = hash_value(
                b"record",
                &(
                    slot as u64,
                    // Physical row order remains part of the canonical storage
                    // grammar: compaction changes this root, allocation IDs do not.
                    physical as u64,
                    arena.partition_ids[physical],
                    arena.generations[physical],
                    arena.lifecycle[physical],
                    arena.kind_ids[physical],
                    history.digest(),
                    arena.created_at[physical],
                    arena.retired_at[physical],
                    extra(&arena.extra[physical], symbols)?,
                    versions,
                ),
            )?;
            hashed_values += 1;
            self.history_bytes = self
                .history_bytes
                .checked_sub(old_bytes)
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?
                .checked_add(history.allocation_bytes())
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
            self.histories.insert(slot as u64, history);
            self.records.set(slot as u128, Some(digest))?;
        }
        for (word, value) in arena
            .live_bitset
            .changed_words_since(Some(&previous.live_bitset))
        {
            self.live_words.set(
                word as u128,
                value
                    .map(|value| hash_value(b"live-word", &value))
                    .transpose()?,
            )?;
            hashed_values += u64::from(value.is_some());
        }
        for (word, value) in arena
            .reclaimable_bitset
            .changed_words_since(Some(&previous.reclaimable_bitset))
        {
            self.reclaimable_words.set(
                word as u128,
                value
                    .map(|value| hash_value(b"reclaimable-word", &value))
                    .transpose()?,
            )?;
            hashed_values += u64::from(value.is_some());
        }
        Ok(hashed_values)
    }

    pub(super) fn digest(&self) -> [u8; 32] {
        hash_value(
            b"record-arena",
            &(
                self.records.digest(),
                self.live_words.digest(),
                self.reclaimable_words.digest(),
            ),
        ).expect(/* Proof: this hash serializes only a fixed tuple of three [u8; 32] arrays into the infallible Sha256Writer. No input-sized collection or custom serializer occurs in this concrete tuple's Serialize implementation. */ "fixed native digest tuple is serializable")
    }
    pub(super) fn allocation_bytes(&self) -> u64 {
        self.records.allocation_bytes()
            + self.histories.allocation_bytes()
            + self.history_bytes
            + self.live_words.allocation_bytes()
            + self.reclaimable_words.allocation_bytes()
    }
    pub(super) fn visit_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        self.records.visit_allocations(visitor);
        self.live_words.visit_allocations(visitor);
        self.reclaimable_words.visit_allocations(visitor);
        self.histories
            .visit_allocations(false, visitor, &mut |history, allocation, visitor| {
                if visitor.visit(allocation) {
                    history.visit_allocations(visitor);
                }
            });
    }
}

fn resolve_aspect_versions<'a>(
    versions: &BTreeMap<Symbol, u64>,
    symbols: &'a StringInterner,
) -> Result<Vec<(&'a str, u64)>, PartitionContentDigestError> {
    let mut resolved = versions
        .iter()
        .map(|(symbol, version)| {
            symbols.resolve(*symbol).map(|name| (name, *version)).ok_or(
                PartitionContentDigestError::UnresolvedContentSymbol(*symbol),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    resolved.sort_unstable_by(|left, right| left.0.cmp(right.0));
    Ok(resolved)
}
