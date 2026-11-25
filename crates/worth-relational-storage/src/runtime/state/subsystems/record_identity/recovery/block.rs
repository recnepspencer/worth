//! Stage one durable commit's live and cancelled allocation evidence (H0, D2).
//!
//! A coupled candidate allocates member by member, cancels superseded slots
//! and records its live evidence in merged-plan order, so a commit's live
//! append slots need not ascend. Together with its cancelled append slots they
//! still fill one contiguous block per class and partition from the frontier
//! (a leading gap is the burned remainder of abandoned candidates). Staging
//! checks that block exactly, burns the cancelled slots, and leaves each live
//! slot to `reserve_exact`, in any order, inside the block.
use std::collections::{BTreeMap, BTreeSet};

use super::super::record_ref::{record_generation, record_partition, record_slot};
use super::super::{RecordIdentitySubsystem, StagedReplayAllocations};
use crate::history::data::canonical_record_allocation::{
    CancelledRecordAllocation, CanonicalRecordAllocation, RecordAllocationOrigin,
};
use crate::transactions::data::outcomes::conflict_class::record_allocation_denial::RecordAllocationDenial;

impl RecordIdentitySubsystem {
    /// Stage `live` and `cancelled` for the next replayed commit. Nothing is
    /// changed unless the whole evidence is admitted. O(evidence · log slots).
    pub fn stage_replay_allocations(
        &self,
        live: Vec<CanonicalRecordAllocation>,
        cancelled: Vec<CancelledRecordAllocation>,
    ) -> Result<(), RecordAllocationDenial> {
        let mut state = self.lock();
        if let Some(staged) = &state.staged_replay_allocations {
            return Err(RecordAllocationDenial::ReplayEvidenceRemaining {
                remaining: staged.live.len() + staged.cancelled.len(),
            });
        }
        // A reservation a checkpoint captured in flight is restored pending;
        // its evidence is checked against that reservation, not the block.
        let restored =
            |class, record: &crate::transactions::data::primitives::RecordRef, origin| {
                state
                    .pending
                    .get(&(class, record_partition(record), record_slot(record)))
                    .is_some_and(|pending| {
                        pending.generation == record_generation(record) && pending.origin == origin
                    })
            };
        let appends = live
            .iter()
            .map(|entry| (entry.class(), entry.record(), entry.origin()))
            .chain(
                cancelled
                    .iter()
                    .map(|entry| (entry.class(), entry.record(), entry.origin())),
            )
            .filter(|(_, _, origin)| *origin == RecordAllocationOrigin::AppendFrontier)
            .filter(|(class, record, origin)| !restored(*class, record, *origin));
        let mut slots = BTreeMap::<_, Vec<usize>>::new();
        for (class, record, _) in appends {
            slots
                .entry((class, record_partition(record)))
                .or_default()
                .push(record_slot(record));
        }
        let mut blocks = BTreeMap::new();
        for ((class, partition_id), mut slots) in slots {
            slots.sort_unstable();
            let frontier = state
                .next_slots
                .get(&(class, partition_id))
                .copied()
                .unwrap_or(0);
            let broken =
                |expected_slot, observed_slot| RecordAllocationDenial::ReplayEvidenceBlockBroken {
                    class,
                    partition_id,
                    expected_slot,
                    observed_slot,
                };
            if slots[0] < frontier {
                return Err(broken(frontier, slots[0]));
            }
            for pair in slots.windows(2) {
                if pair[1] != pair[0] + 1 {
                    return Err(broken(pair[0] + 1, pair[1]));
                }
            }
            blocks.insert((class, partition_id), slots[0]..slots[slots.len() - 1] + 1);
        }
        let mut burned = BTreeSet::new();
        let mut released = Vec::new();
        for entry in &cancelled {
            let record = entry.record();
            let key = (entry.class(), record_partition(record), record_slot(record));
            if restored(entry.class(), record, entry.origin()) {
                if !burned.insert(key) {
                    return Err(RecordAllocationDenial::ReplayCancelledSlotInvalid {
                        class: key.0,
                        partition_id: key.1,
                        slot: key.2,
                        expected_generation: record_generation(record),
                        observed_generation: record_generation(record),
                    });
                }
                released.push((key, entry.origin()));
                continue;
            }
            let held = state.generation_high_water.get(&key).copied().unwrap_or(0);
            let generation = record_generation(record);
            let prior = match entry.origin() {
                RecordAllocationOrigin::AppendFrontier => 0,
                RecordAllocationOrigin::Reclaimed { prior_generation } => prior_generation,
            };
            let reusable = matches!(entry.origin(), RecordAllocationOrigin::AppendFrontier)
                || state.reusable.contains_key(&key);
            if held != prior
                || prior.checked_add(1) != Some(generation)
                || !reusable
                || !burned.insert(key)
            {
                return Err(RecordAllocationDenial::ReplayCancelledSlotInvalid {
                    class: key.0,
                    partition_id: key.1,
                    slot: key.2,
                    expected_generation: generation,
                    observed_generation: held,
                });
            }
        }
        for (key, block) in &blocks {
            let frontier = state.next_slots.entry(*key).or_default();
            *frontier = (*frontier).max(block.end);
        }
        for entry in &cancelled {
            let record = entry.record();
            let key = (entry.class(), record_partition(record), record_slot(record));
            state
                .generation_high_water
                .insert(key, record_generation(record));
        }
        // A cancelled in-flight reservation is released as its live drop did.
        for (key, origin) in released {
            state.pending.remove(&key);
            if matches!(origin, RecordAllocationOrigin::Reclaimed { .. }) {
                state.reusable.insert(key, ());
            }
        }
        state.replay_blocks = blocks;
        state.staged_replay_allocations = Some(StagedReplayAllocations { live, cancelled });
        Ok(())
    }
}
