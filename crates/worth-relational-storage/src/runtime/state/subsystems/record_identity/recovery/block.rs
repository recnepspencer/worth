//! Stage one durable commit's live and cancelled allocation evidence (H0, D2,
//! #389).
//!
//! A coupled candidate allocates member by member, cancels superseded slots
//! and records its live evidence in merged-plan order, so a commit's live
//! append slots need not ascend. Concurrent branch candidates also reserve
//! interleaved append slots and may publish out of preparation order, so a
//! commit's slots need not be contiguous nor above the frontier. Staging
//! walks the commit's live and cancelled append slots in ascending order: a
//! slot at or above the frontier records the slots it skips as replay gaps
//! and advances the frontier; a slot below it must consume a recorded gap.
//! A duplicate, or a slot below the frontier that no gap holds, is refused.
//! Staging burns the cancelled slots and leaves each live slot to
//! `reserve_exact`, in any order, among the staged ones.
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
            .map(|entry| (entry.class(), entry.record(), entry.origin(), true))
            .chain(
                cancelled
                    .iter()
                    .map(|entry| (entry.class(), entry.record(), entry.origin(), false)),
            )
            .filter(|(_, _, origin, _)| *origin == RecordAllocationOrigin::AppendFrontier)
            .filter(|(class, record, origin, _)| !restored(*class, record, *origin));
        let mut slots = BTreeMap::<_, Vec<(usize, bool)>>::new();
        for (class, record, _, live) in appends {
            slots
                .entry((class, record_partition(record)))
                .or_default()
                .push((record_slot(record), live));
        }
        let mut gaps = state.replay_gaps.clone();
        let mut frontiers = BTreeMap::new();
        let mut staged = BTreeMap::<_, BTreeSet<usize>>::new();
        for ((class, partition_id), mut slots) in slots {
            let key = (class, partition_id);
            slots.sort_unstable();
            let mut frontier = state.next_slots.get(&key).copied().unwrap_or(0);
            let broken =
                |expected_slot, observed_slot| RecordAllocationDenial::ReplayEvidenceBlockBroken {
                    class,
                    partition_id,
                    expected_slot,
                    observed_slot,
                };
            let mut previous = None;
            for (slot, live) in slots {
                if previous == Some(slot) {
                    return Err(broken(slot + 1, slot));
                }
                previous = Some(slot);
                if slot < frontier {
                    if !gaps.take(key, slot) {
                        return Err(broken(frontier, slot));
                    }
                } else {
                    gaps.skip(key, frontier, slot);
                    frontier = slot.checked_add(1).ok_or(
                        RecordAllocationDenial::SlotFrontierExhausted {
                            class,
                            partition_id,
                        },
                    )?;
                }
                if live {
                    staged.entry(key).or_default().insert(slot);
                }
            }
            frontiers.insert(key, frontier);
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
        for (key, frontier) in frontiers {
            state.next_slots.insert(key, frontier);
        }
        state.replay_gaps = gaps;
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
        state.replay_slots = staged;
        state.staged_replay_allocations = Some(StagedReplayAllocations { live, cancelled });
        Ok(())
    }
}
