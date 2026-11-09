//! Whole arena construction, including authenticated Head frontier seeds.
use super::*;
impl RecordContentIndex {
    pub(in crate::storage::overlay::partition_content) fn rebuild<K: RecordKind>(
        &mut self,
        lane: &ReconstructionLane<'_>,
        work: &mut ReconstructionWork<'_, '_>,
        request: super::super::request_admission::Request<'_, '_, '_>,
        arena: &RecordArena<K>,
        symbols: &dyn crate::symbols::data::SymbolTextLookup,
        extra: impl Fn(
            &K::Extra,
            &dyn crate::symbols::data::SymbolTextLookup,
            &mut ReconstructionWork<'_, '_>,
        ) -> Result<[u8; 32], PartitionContentDigestError>,
    ) -> Result<u64, PartitionContentDigestError>
    where
        K::Meta: serde::Serialize,
    {
        let capacity = arena.generations.len();
        work.prepare(
            capacity as u64,
            capacity as u64
                * (size_of::<(u128, [u8; 32])>() + size_of::<(u64, DigestIndex)>()) as u64,
        )?;
        let mut entries = Vec::with_capacity(capacity);
        let mut histories = Vec::with_capacity(capacity);
        let mut hashed = 0;
        for slot in arena.occupied_slots_iter() {
            work.prepare(1, 0)?;
            let physical = arena
                .physical_index(slot)
                .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
            let cached = self.histories.get(&(slot as u64)).cloned();
            let old_bytes = cached.as_ref().map_or(0, DigestIndex::allocation_bytes);
            let (history, count) = match cached {
                Some(carried) => (carried, 0),
                None => super::super::record_history::rebuild(lane, work, request, arena, slot)?,
            };
            hashed += count + 1;
            work.prepare(arena.aspect_versions[physical].len() as u64, 0)?;
            let versions =
                super::resolve_aspect_versions(&arena.aspect_versions[physical], symbols, request)?;
            let digest = super::super::admission::hash_controlled(
                b"record",
                &(
                    slot as u64,
                    physical as u64,
                    arena.partition_ids[physical],
                    arena.generations[physical],
                    arena.lifecycle[physical],
                    arena.kind_ids[physical],
                    history.digest(),
                    arena.created_at[physical],
                    arena.retired_at[physical],
                    extra(&arena.extra[physical], symbols, work)?,
                    versions,
                ),
                work,
            )?;
            self.history_bytes = self
                .history_bytes
                .checked_sub(old_bytes)
                .and_then(|n| n.checked_add(history.allocation_bytes()))
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
            histories.push((slot as u64, history));
            entries.push((slot as u128, digest));
        }
        work.prepare(
            (entries.len() * (entries.len().max(1).ilog2() as usize + 1)) as u64,
            0,
        )?;
        entries.sort_unstable_by_key(|(slot, _)| *slot);
        histories.sort_unstable_by_key(|(slot, _)| *slot);
        let (units, _) = DigestIndex::construction_quote(entries.len())
            .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
        work.prepare(units, 0)?;
        self.records = DigestIndex::from_sorted_entries_for_request(lane, &entries, request)?;
        work.prepare(histories.len() as u64, 0)?;
        for (slot, history) in histories {
            match request {
                Some(request) => {
                    self.histories
                        .insert_for_request(slot, history, request)
                        .map_err(PartitionContentDigestError::Request)?;
                }
                None => {
                    self.histories.insert(slot, history);
                }
            }
        }
        for (bitset, index, domain) in [
            (
                &arena.live_bitset,
                &mut self.live_words,
                b"live-word".as_slice(),
            ),
            (
                &arena.reclaimable_bitset,
                &mut self.reclaimable_words,
                b"reclaimable-word".as_slice(),
            ),
        ] {
            let words = bitset.stored_word_count();
            work.prepare(
                words as u64,
                (words * (size_of::<(u64, u64)>() + size_of::<(u128, [u8; 32])>())) as u64,
            )?;
            let mut entries = Vec::with_capacity(words);
            for (word, value) in bitset.sparse_words() {
                entries.push((
                    word as u128,
                    super::super::admission::hash_controlled(domain, &value, work)?,
                ));
                hashed += 1;
            }
            let (units, _) = DigestIndex::construction_quote(entries.len())
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
            work.prepare(units, 0)?;
            *index = DigestIndex::from_sorted_entries_for_request(lane, &entries, request)?;
        }
        Ok(hashed)
    }
    pub(in crate::storage::overlay::partition_content) fn from_history_frontiers<K: RecordKind>(
        _lane: &ReconstructionLane<'_>,
        work: &mut ReconstructionWork<'_, '_>,
        request: super::super::request_admission::Request<'_, '_, '_>,
        frontiers: &[(u64, HistoryAppendFrontier)],
        arena: &RecordArena<K>,
    ) -> Result<Self, PartitionContentDigestError>
    where
        K::Meta: serde::Serialize,
    {
        let invalid = || PartitionContentDigestError::InvalidHistoryCoverage;
        work.prepare(frontiers.len() as u64, 0)?;
        if !frontiers
            .iter()
            .map(|(slot, _)| *slot)
            .eq(arena.occupied_slots_iter().map(|slot| slot as u64))
        {
            return Err(invalid());
        }
        let mut result = Self::default();
        for (slot, frontier) in frontiers {
            let (units, _) = DigestIndex::construction_quote(
                frontier.siblings.len().checked_add(1).ok_or_else(invalid)?,
            )
            .ok_or_else(invalid)?;
            work.prepare(units, 0)?;
            let history = DigestIndex::from_append_frontier_using(frontier, request)?;
            let metadata = arena
                .metadata_history_at(usize::try_from(*slot).map_err(|_| invalid())?)
                .ok_or_else(invalid)?;
            if metadata.len() != usize::from(history.next_history_index().ok_or_else(invalid)? > 0)
            {
                return Err(invalid());
            }
            let last = metadata
                .get(0)
                .map(|value| super::super::admission::hash_controlled(b"metadata", value, work))
                .transpose()?;
            history.append_frontier(last).ok_or_else(invalid)?;
            result.history_bytes += history.allocation_bytes();
            if result.histories.contains_key(slot) {
                return Err(invalid());
            }
            let bytes = result
                .histories
                .adoption_allocation_bound()
                .ok_or_else(invalid)?;
            work.prepare(
                result.histories.navigation_work_bound() as u64,
                if request.is_some() { 0 } else { bytes },
            )?;
            match request {
                Some(request) => {
                    result
                        .histories
                        .insert_for_request(*slot, history, request)
                        .map_err(PartitionContentDigestError::Request)?;
                }
                None => {
                    result.histories.insert(*slot, history);
                }
            }
        }
        Ok(result)
    }
}
