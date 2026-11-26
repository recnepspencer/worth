use super::arena_generation_ledger::{
    arena_range, guard_generation, guard_range, physical_generation,
};
use super::*;
use crate::allocation_generation_guard::{
    active_generation, can_admit_for_reuse, frame_lookup_matches, ranges_overlap, GuardIdentity,
    GuardRange, ReuseBarrier,
};
use crate::{ArenaReuseBarrier, WritebackFenceDrainDenial};
use worth_store_physical_format::{
    ArenaExtentFrameIdentity, ExtentAllocationGeneration, ExtentArenaRange,
};

impl PhysicalResidencyPool {
    /// `WritebackFenceDrain`: the only door that returns an arena range to
    /// allocator reuse. It fences new writeback claims overlapping `range`,
    /// refuses while any overlapping claim is in flight or any overlapping
    /// frame is not clean and unpinned, then evicts the overlapping clean
    /// frames and records `next` as the range's generation, all under one pool
    /// lock. `next` must exceed every generation seen on an overlapping range.
    pub fn drain_writeback_fence(
        &self,
        range: ExtentArenaRange,
        barrier: ArenaReuseBarrier,
        next: ExtentAllocationGeneration,
    ) -> Result<ArenaExtentFrameIdentity, WritebackFenceDrainDenial> {
        let mut state = self.inner.lock();
        if !state.accepting {
            state.accounting.deny();
            return Err(WritebackFenceDrainDenial::PoolClosed);
        }
        let guard_barrier = match barrier {
            ArenaReuseBarrier::RetirementIncomplete => ReuseBarrier::RetirementIncomplete,
            ArenaReuseBarrier::CompletedRetirement => ReuseBarrier::CompletedRetirement,
            ArenaReuseBarrier::CancelUncovered => ReuseBarrier::CancelUncovered,
            ArenaReuseBarrier::CheckpointCoveredCancel => ReuseBarrier::CheckpointCoveredCancel,
        };
        let freed = guard_range(range);
        let overlapping = overlapping_coordinates(&state, freed);
        let in_flight: Vec<GuardRange> = overlapping
            .iter()
            .filter(|coordinate| {
                state
                    .frames
                    .get(coordinate)
                    .expect("occupied slot")
                    .writeback_claimed
            })
            .filter_map(|coordinate| arena_range(*coordinate).map(guard_range))
            .collect();
        let seen = state.arena_generations.overlapping(freed);
        let admissible = can_admit_for_reuse(
            freed,
            &in_flight,
            guard_barrier,
            &seen,
            guard_generation(next),
        );
        if !admissible
            && !matches!(
                barrier,
                ArenaReuseBarrier::CompletedRetirement | ArenaReuseBarrier::CheckpointCoveredCancel
            )
        {
            state.accounting.deny();
            return Err(WritebackFenceDrainDenial::BarrierIncomplete(barrier));
        }
        if !admissible {
            state.accounting.deny();
            if !in_flight.is_empty() {
                // Only fence when waiting on in-flight writebacks (never on a
                // terminal `GenerationNotAdvanced` refusal).
                state.arena_generations.fence(range);
                return Err(WritebackFenceDrainDenial::WritebackInFlight {
                    overlapping: u32::try_from(in_flight.len()).unwrap_or(u32::MAX),
                });
            }
            return Err(WritebackFenceDrainDenial::GenerationNotAdvanced {
                proposed: next,
                active: physical_generation(active_generation(freed, &seen)),
            });
        }
        let artifact = RecordArtifactFile::ExtentArena {
            arena: range.arena().get(),
        };
        let busy = if state.frames.contains_artifact_alias(artifact) {
            Some(PhysicalResidencyDenial::ArtifactIdentityOccupied)
        } else {
            overlapping.iter().find_map(|coordinate| {
                validate_drain_eviction(state.frames.get(coordinate).expect("occupied slot")).err()
            })
        };
        if let Some(reason) = busy {
            state.arena_generations.fence(range);
            state.accounting.deny();
            return Err(WritebackFenceDrainDenial::FrameBusy(reason));
        }
        for coordinate in overlapping {
            let was_dirty = state
                .frames
                .get(&coordinate)
                .expect("validated frame")
                .dirty;
            if !was_dirty {
                state.detach_evictable(coordinate);
            }
            let removed = state.frames.remove(&coordinate).expect("validated frame");
            state.accounting.remove_frame(removed.accounting_removal());
            if was_dirty {
                state.accounting.record_administrative_drain();
            }
        }
        state.arena_generations.admit(range, next);
        self.inner.changed.notify_all();
        Ok(ArenaExtentFrameIdentity::new(range, next))
    }

    /// Seeds the durable reopen frontier floor so post-reopen arena frames
    /// and active-generation queries never rewind below pre-crash barriers.
    pub fn seed_arena_generation_floor(&self, floor: ExtentAllocationGeneration) {
        let mut state = self.inner.lock();
        state.arena_generations.seed_floor(floor);
    }

    /// The newest generation admitted on any range overlapping `range` in this
    /// incarnation, or [`ExtentAllocationGeneration::OPENED`].
    pub fn active_arena_generation(&self, range: ExtentArenaRange) -> ExtentAllocationGeneration {
        let state = self.inner.lock();
        let guard = guard_range(range);
        let active = active_generation(guard, &state.arena_generations.overlapping(guard));
        physical_generation(active)
    }

    /// The allocation identity a resident arena frame carries.
    pub fn arena_frame_identity(&self, key: PhysicalFrameKey) -> Option<ArenaExtentFrameIdentity> {
        let state = self.inner.lock();
        state
            .frames
            .get(&key.coordinate)?
            .arena_identity(key.coordinate)
    }

    /// Serves an arena frame only at the exact identity requested. A frame
    /// admitted under an older generation, or a request naming one, is refused.
    pub fn require_arena_frame_identity(
        &self,
        key: PhysicalFrameKey,
        expected: ArenaExtentFrameIdentity,
    ) -> Result<(), PhysicalResidencyDenial> {
        self.inner.validate_key(key)?;
        let mut state = self.inner.lock();
        let Some(entry) = state.frames.get(&key.coordinate) else {
            return Err(PoolInner::deny(
                &mut state,
                PhysicalResidencyDenial::FrameNotResident,
            ));
        };
        let current = state.arena_generations.identity_for(key.coordinate);
        let matches = match (entry.arena_identity(key.coordinate), current) {
            (Some(frame), Some(current)) => {
                frame == current
                    && frame_lookup_matches(
                        guard_identity(frame),
                        guard_range(expected.range()),
                        guard_generation(expected.generation()),
                    )
            }
            _ => false,
        };
        if matches {
            Ok(())
        } else {
            Err(PoolInner::deny(
                &mut state,
                PhysicalResidencyDenial::StaleArenaGeneration,
            ))
        }
    }
}

fn validate_drain_eviction(entry: &FrameEntry) -> Result<(), PhysicalResidencyDenial> {
    match &entry.state {
        FrameState::LoadFailed(terminal) => {
            Err(PhysicalResidencyDenial::FrameLoadTerminated(*terminal))
        }
        FrameState::Loading | FrameState::CandidateReserved => {
            Err(PhysicalResidencyDenial::FrameIdentityOccupied)
        }
        FrameState::Resident(_) if entry.pins != 0 => Err(PhysicalResidencyDenial::FramePinned),
        FrameState::Resident(_) if entry.writeback_claimed => {
            Err(PhysicalResidencyDenial::WriteBackFrameAlreadyClaimed)
        }
        // After a durable reuse barrier, unpinned unclaimed dirty frames hold
        // obsolete bytes for the retired/cancelled range and are discarded.
        FrameState::Resident(_) => Ok(()),
    }
}

fn overlapping_coordinates(state: &PoolState, freed: GuardRange) -> Vec<RecordFrameCoordinate> {
    (0..state.frames.slot_count())
        .filter_map(|slot| state.frames.coordinate_at(slot))
        .filter(|coordinate| {
            arena_range(*coordinate).is_some_and(|range| ranges_overlap(freed, guard_range(range)))
        })
        .collect()
}

const fn guard_identity(identity: ArenaExtentFrameIdentity) -> GuardIdentity {
    GuardIdentity {
        range: guard_range(identity.range()),
        generation: guard_generation(identity.generation()),
    }
}
