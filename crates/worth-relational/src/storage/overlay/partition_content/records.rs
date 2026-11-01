mod frontier_view;
use super::{digest_index::DigestIndex, hash_value, PartitionContentDigestError};
use crate::storage::substrate::{
    HistoryAppendFrontier, RecordArena, RecordKind, SharedMap, StorageAllocationVisitor,
};
use crate::symbols::data::SymbolTextLookup;

#[derive(Debug, Clone, Default)]
pub(super) struct RecordContentIndex {
    records: DigestIndex,
    histories: SharedMap<u64, DigestIndex>,
    history_bytes: u64,
    live_words: DigestIndex,
    reclaimable_words: DigestIndex,
}

impl RecordContentIndex {
    pub(super) fn capture_full<K: RecordKind>(
        arena: &RecordArena<K>,
        symbols: &dyn SymbolTextLookup,
        request: super::full::Request<'_, '_, '_>,
        extra: impl Fn(
            &K::Extra,
            &dyn SymbolTextLookup,
        ) -> Result<[u8; 32], PartitionContentDigestError>,
    ) -> Result<(Self, u64), PartitionContentDigestError>
    where
        K::Meta: serde::Serialize,
    {
        let mut count = 0usize;
        for _ in arena.occupied_slots_iter() {
            super::full::poll(request)?;
            count = count
                .checked_add(1)
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
        }
        let _memory = super::full::rows_memory(count, request)?;
        let mut rows = Vec::with_capacity(count);
        let mut result = Self::default();
        let mut hashed = 0u64;
        for slot in arena.occupied_slots_iter() {
            super::full::poll(request)?;
            let physical = arena
                .physical_index(slot)
                .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
            let (history, values) = super::record_history::capture_full(arena, slot, request)?;
            rows.push((
                slot as u128,
                record_digest(
                    arena,
                    slot,
                    physical,
                    history.digest(),
                    symbols,
                    &extra,
                    request,
                )?,
            ));
            result.history_bytes = result
                .history_bytes
                .checked_add(history.allocation_bytes())
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
            match request {
                Some(request) => {
                    result
                        .histories
                        .insert_for_request(slot as u64, history, request)
                        .map_err(PartitionContentDigestError::Request)?;
                }
                None => {
                    result.histories.insert(slot as u64, history);
                }
            }
            hashed = hashed
                .checked_add(values)
                .and_then(|sum| sum.checked_add(1))
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
        }
        rows.sort_unstable_by_key(|(key, _)| *key);
        result.records = DigestIndex::from_sorted_entries_for_request(&rows, request)?;
        for (words, target, domain) in [
            (
                &arena.live_bitset,
                &mut result.live_words,
                b"live-word".as_slice(),
            ),
            (
                &arena.reclaimable_bitset,
                &mut result.reclaimable_words,
                b"reclaimable-word".as_slice(),
            ),
        ] {
            let mut count = 0usize;
            for _ in words.words() {
                super::full::poll(request)?;
                count = count
                    .checked_add(1)
                    .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
            }
            let _memory = super::full::rows_memory(count, request)?;
            let mut rows = Vec::with_capacity(count);
            for (index, value) in words.words() {
                super::full::poll(request)?;
                rows.push((index as u128, hash_value(domain, &value)?));
            }
            *target = DigestIndex::from_sorted_entries_for_request(&rows, request)?;
            hashed = hashed
                .checked_add(rows.len() as u64)
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
        }
        Ok((result, hashed))
    }

    pub(super) fn history_frontiers<K: RecordKind>(
        &self,
        arena: &RecordArena<K>,
    ) -> Option<Vec<(u64, HistoryAppendFrontier)>>
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
                    .map(|value| hash_value(b"metadata", value))
                    .transpose()
                    .ok()?;
                Some((slot, history.append_frontier(last_value)?))
            })
            .collect()
    }

    pub(super) fn from_history_frontiers<K: RecordKind>(
        frontiers: &[(u64, HistoryAppendFrontier)],
        arena: &RecordArena<K>,
    ) -> Option<Self>
    where
        K::Meta: serde::Serialize,
    {
        if !frontiers
            .iter()
            .map(|(slot, _)| *slot)
            .eq(arena.occupied_slots_iter().map(|slot| slot as u64))
        {
            return None;
        }
        let mut result = Self::default();
        for (slot, frontier) in frontiers {
            let history = DigestIndex::from_append_frontier(frontier)?;
            let metadata = arena.metadata_history_at(usize::try_from(*slot).ok()?)?;
            if metadata.len() != usize::from(history.next_history_index()? > 0) {
                return None;
            }
            let last = metadata
                .get(0)
                .map(|value| hash_value(b"metadata", value))
                .transpose()
                .ok()?;
            history.append_frontier(last)?;
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
        symbols: &dyn SymbolTextLookup,
        extra: impl Fn(
            &K::Extra,
            &dyn SymbolTextLookup,
        ) -> Result<[u8; 32], PartitionContentDigestError>,
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
                .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
            let carried_head = previous.is_none() && self.histories.contains_key(&(slot as u64));
            let cached = self
                .histories
                .get(&(slot as u64))
                .cloned()
                .unwrap_or_default();
            let old_bytes = cached.allocation_bytes();
            let (history, hashed) = if carried_head {
                (cached, 0)
            } else {
                super::record_history::update(arena, previous, slot, &cached)?
            };
            hashed_values += hashed;
            let digest = record_digest(
                arena,
                slot,
                physical,
                history.digest(),
                symbols,
                &extra,
                None,
            )?;
            hashed_values += 1;
            self.history_bytes = self
                .history_bytes
                .checked_sub(old_bytes)
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?
                .checked_add(history.allocation_bytes())
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
            if let Some(entries) = &mut cold_histories {
                entries.push((slot as u64, history));
            } else {
                self.histories.insert(slot as u64, history);
            }
            if let Some(entries) = &mut cold_records {
                entries.push((slot as u128, digest));
            } else {
                self.records.set(slot as u128, Some(digest))?;
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
                value
                    .map(|value| hash_value(b"live-word", &value))
                    .transpose()?,
            )?;
            hashed_values += u64::from(value.is_some());
        }
        for (word, value) in arena
            .reclaimable_bitset
            .changed_words_since(previous.map(|old| &old.reclaimable_bitset))
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
            .visit_allocations(false, visitor, &mut |history, _, visitor| {
                history.visit_allocations(visitor);
            });
    }
}

fn record_digest<K: RecordKind>(
    arena: &RecordArena<K>,
    slot: usize,
    physical: usize,
    history: [u8; 32],
    symbols: &dyn SymbolTextLookup,
    extra: &impl Fn(&K::Extra, &dyn SymbolTextLookup) -> Result<[u8; 32], PartitionContentDigestError>,
    request: super::full::Request<'_, '_, '_>,
) -> Result<[u8; 32], PartitionContentDigestError> {
    let versions = &arena.aspect_versions[physical];
    let bytes = std::alloc::Layout::array::<(&str, u64)>(versions.len())
        .ok()
        .and_then(|layout| u64::try_from(layout.size()).ok())
        .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
    let _memory = request
        .map(|request| {
            request
                .reserve_retained_memory(bytes)
                .map_err(|cause| PartitionContentDigestError::Request(cause.into()))
        })
        .transpose()?
        .flatten();
    let mut resolved = Vec::with_capacity(versions.len());
    for (symbol, version) in versions.iter() {
        super::full::poll(request)?;
        let name = symbols.resolve(*symbol).ok_or(
            PartitionContentDigestError::UnresolvedContentSymbol(*symbol),
        )?;
        resolved.push((name, *version));
    }
    resolved.sort_unstable_by(|left, right| left.0.cmp(right.0));
    hash_value(
        b"record",
        &(
            slot as u64,
            physical as u64,
            arena.partition_ids[physical],
            arena.generations[physical],
            arena.lifecycle[physical],
            arena.kind_ids[physical],
            history,
            arena.created_at[physical],
            arena.retired_at[physical],
            extra(&arena.extra[physical], symbols)?,
            resolved,
        ),
    )
}
