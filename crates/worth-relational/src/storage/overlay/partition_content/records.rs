use super::{digest_index::DigestIndex, hash_value, PartitionContentDigestError};
use crate::storage::substrate::{
    RecordArena, RecordKind, SharedColumn, SharedMap, StorageAllocationVisitor,
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
        arena: &RecordArena<K>,
    ) -> Option<Vec<(u64, super::digest_index::HistoryAppendFrontier)>>
    where
        K::Meta: serde::Serialize,
    {
        self.histories
            .iter()
            .map(|(&slot, history)| {
                let metadata = arena.metadata_history_at(usize::try_from(slot).ok()?)?;
                let last_value = metadata
                    .len()
                    .checked_sub(1)
                    .and_then(|index| metadata.get(index))
                    .map(|value| hash_value(b"metadata", value));
                Some((slot, history.append_frontier(last_value)?))
            })
            .collect()
    }

    pub(super) fn from_history_frontiers(
        frontiers: &[(u64, super::digest_index::HistoryAppendFrontier)],
    ) -> Option<Self> {
        let mut result = Self::default();
        for (slot, frontier) in frontiers {
            let history = DigestIndex::from_append_frontier(frontier)?;
            result.history_bytes += history.allocation_bytes();
            if result.histories.contains_key(slot) {
                return None;
            }
            result.histories.insert(*slot, history);
        }
        Some(result)
    }

    pub(super) fn update<K: RecordKind>(
        &mut self,
        arena: &RecordArena<K>,
        previous: Option<&RecordArena<K>>,
        slots: impl IntoIterator<Item = usize>,
        symbols: &StringInterner,
        extra: impl Fn(&K::Extra, &StringInterner) -> Result<[u8; 32], PartitionContentDigestError>,
    ) -> Result<u64, PartitionContentDigestError>
    where
        K::Meta: serde::Serialize,
    {
        let mut hashed_values = 0;
        let mut cold_records = previous.is_none().then(Vec::new);
        let mut cold_histories = previous.is_none().then(Vec::new);
        for slot in slots {
            let physical = arena
                .physical_index(slot)
                .expect("journal-selected record exists");
            let mut history = self
                .histories
                .get(&(slot as u64))
                .cloned()
                .unwrap_or_default();
            let old_bytes = history.allocation_bytes();
            let empty = SharedColumn::default();
            let old = previous
                .and_then(|arena| arena.metadata_history_at(slot))
                .unwrap_or(&empty);
            let history_offset = if previous.is_some() {
                history
                    .next_history_index()
                    .checked_sub(old.len() as u128)
                    .expect("resident history suffix belongs to its carried commitment")
            } else {
                0
            };
            arena.metadata_history[physical].visit_changed_values(old, &mut |index, value, _| {
                history.set(
                    history_offset + index as u128,
                    value.map(|value| hash_value(b"metadata", value)),
                );
                hashed_values += u64::from(value.is_some());
            });
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
            );
            hashed_values += 1;
            self.history_bytes = self
                .history_bytes
                .checked_sub(old_bytes)
                .unwrap()
                .checked_add(history.allocation_bytes())
                .unwrap();
            if let Some(entries) = &mut cold_histories {
                entries.push((slot as u64, history));
            } else {
                self.histories.insert(slot as u64, history);
            }
            if let Some(entries) = &mut cold_records {
                entries.push((slot as u128, digest));
            } else {
                self.records.set(slot as u128, Some(digest));
            }
        }
        if let Some(mut entries) = cold_records {
            entries.sort_unstable_by_key(|(slot, _)| *slot);
            self.records = DigestIndex::from_sorted_entries(&entries);
        }
        if let Some(mut entries) = cold_histories {
            entries.sort_unstable_by_key(|(slot, _)| *slot);
            self.histories = SharedMap::from_sorted_unique(entries);
        }
        for (word, value) in arena
            .live_bitset
            .changed_words_since(previous.map(|old| &old.live_bitset))
        {
            self.live_words.set(
                word as u128,
                value.map(|value| hash_value(b"live-word", &value)),
            );
            hashed_values += u64::from(value.is_some());
        }
        for (word, value) in arena
            .reclaimable_bitset
            .changed_words_since(previous.map(|old| &old.reclaimable_bitset))
        {
            self.reclaimable_words.set(
                word as u128,
                value.map(|value| hash_value(b"reclaimable-word", &value)),
            );
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
        )
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
