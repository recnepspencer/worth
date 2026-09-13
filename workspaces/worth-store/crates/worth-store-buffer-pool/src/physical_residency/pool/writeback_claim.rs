use super::*;

impl PoolInner {
    pub(super) fn claim_writeback(
        self: &Arc<Self>,
        allocation: &ForegroundWriteAllocationGrant,
        frames: &[PhysicalFrameKey],
    ) -> Result<ClaimedWritebackFrames, PhysicalResidencyDenial> {
        self.validate_writeback_frames(frames)?;
        let mut state = self.lock();
        let kind = crate::PhysicalSpeculativeWorkKind::WriteBehind;
        state.accounting.attempt_speculative(kind);
        if !state.accepting {
            state.accounting.record_speculative_denial(kind);
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        let required = match self.validate_writeback_residents(&mut state, frames) {
            Ok(required) => required,
            Err(denial) => {
                state.accounting.record_speculative_denial(kind);
                return Err(denial);
            }
        };
        if allocation.bytes() != required {
            state.accounting.record_speculative_denial(kind);
            return Err(Self::deny(
                &mut state,
                PhysicalResidencyDenial::SpeculativeAllocationMismatch {
                    granted: allocation.bytes(),
                    required,
                },
            ));
        }
        let count = frames.len() as u32;
        self.admit_writeback_capacity(&mut state, count)?;
        Self::apply_writeback_claim(&mut state, frames, count);
        let request = match self.allocate_claim_evidence(&mut state, frames) {
            Ok(request) => request,
            Err(denial) => {
                Self::rollback_writeback_claim(&mut state, frames, count);
                return Err(denial);
            }
        };
        Ok(ClaimedWritebackFrames {
            frames: request.frames,
            resident_bytes: request.resident_bytes,
            range_postures: request.range_postures,
        })
    }

    fn validate_writeback_frames(
        &self,
        frames: &[PhysicalFrameKey],
    ) -> Result<(), PhysicalResidencyDenial> {
        if frames.is_empty() {
            return Err(self.record_denial(PhysicalResidencyDenial::WriteBackExceedsDirtyPosture));
        }
        u32::try_from(frames.len()).map_err(|_| {
            self.record_denial(PhysicalResidencyDenial::WriteBackExceedsDirtyPosture)
        })?;
        for (index, frame) in frames.iter().enumerate() {
            if let Err(reason) = self.validate_key(*frame) {
                return Err(self.record_denial(reason));
            }
            if frames[..index].contains(frame) {
                return Err(
                    self.record_denial(PhysicalResidencyDenial::WriteBackFrameAlreadyClaimed)
                );
            }
        }
        Ok(())
    }

    fn admit_writeback_capacity(
        &self,
        state: &mut PoolState,
        count: u32,
    ) -> Result<(), PhysicalResidencyDenial> {
        let kind = crate::PhysicalSpeculativeWorkKind::WriteBehind;
        let current = state.accounting.active_speculative_frames(kind);
        let next = current.checked_add(count).ok_or_else(|| {
            state.accounting.record_speculative_denial(kind);
            self.pressure(
                state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::SpeculativeFrames(kind),
                    scope: PhysicalOperationAllocationScope::ForegroundWrite,
                    requested: u64::from(count),
                    current: u64::from(current),
                    limit: u64::from(self.limits.speculative_frames(kind)),
                },
            )
        })?;
        if next > self.limits.speculative_frames(kind) {
            state.accounting.record_speculative_denial(kind);
            return Err(self.pressure(
                state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::SpeculativeFrames(kind),
                    scope: PhysicalOperationAllocationScope::ForegroundWrite,
                    requested: u64::from(count),
                    current: u64::from(current),
                    limit: u64::from(self.limits.speculative_frames(kind)),
                },
            ));
        }
        Ok(())
    }

    fn validate_writeback_residents(
        &self,
        state: &mut PoolState,
        frames: &[PhysicalFrameKey],
    ) -> Result<u64, PhysicalResidencyDenial> {
        let mut required = 0_u64;
        for frame in frames {
            let Some(entry) = state.frames.get(&frame.coordinate) else {
                return Err(Self::deny(
                    state,
                    PhysicalResidencyDenial::WriteBackFrameNotDirty,
                ));
            };
            if !entry.dirty {
                return Err(Self::deny(
                    state,
                    PhysicalResidencyDenial::WriteBackFrameNotDirty,
                ));
            }
            if entry.writeback_claimed {
                return Err(Self::deny(
                    state,
                    PhysicalResidencyDenial::WriteBackFrameAlreadyClaimed,
                ));
            }
            let FrameState::Resident(bytes) = &entry.state else {
                return Err(Self::deny(
                    state,
                    PhysicalResidencyDenial::WriteBackFrameNotDirty,
                ));
            };
            required = required
                .checked_add(bytes.len() as u64)
                .ok_or_else(|| Self::deny(state, PhysicalResidencyDenial::AllocationFailed))?;
        }
        Ok(required)
    }

    fn allocate_claim_evidence(
        &self,
        state: &mut PoolState,
        frames: &[PhysicalFrameKey],
    ) -> Result<PreparedWritebackClaim, PhysicalResidencyDenial> {
        let mut owned_frames = Vec::new();
        owned_frames
            .try_reserve_exact(frames.len())
            .map_err(|_| Self::deny(state, PhysicalResidencyDenial::AllocationFailed))?;
        let mut resident_bytes = Vec::new();
        resident_bytes
            .try_reserve_exact(frames.len())
            .map_err(|_| Self::deny(state, PhysicalResidencyDenial::AllocationFailed))?;
        let mut range_postures = Vec::new();
        range_postures
            .try_reserve_exact(frames.len())
            .map_err(|_| Self::deny(state, PhysicalResidencyDenial::AllocationFailed))?;
        for frame in frames {
            let entry = state
                .frames
                .get(&frame.coordinate)
                .expect("claimed writeback frame remains resident");
            let FrameState::Resident(bytes) = &entry.state else {
                unreachable!("validated writeback frame remains resident")
            };
            owned_frames.push(*frame);
            resident_bytes.push(Arc::clone(bytes));
            range_postures.push(entry.origin.writeback_range_posture(frame.coordinate));
        }
        Ok(PreparedWritebackClaim {
            frames: owned_frames.into_boxed_slice(),
            resident_bytes,
            range_postures,
        })
    }

    fn apply_writeback_claim(state: &mut PoolState, frames: &[PhysicalFrameKey], count: u32) {
        let kind = crate::PhysicalSpeculativeWorkKind::WriteBehind;
        for frame in frames {
            state
                .frames
                .get_mut(&frame.coordinate)
                .expect("validated dirty frame remains resident")
                .writeback_claimed = true;
        }
        state.accounting.admit_speculative(kind, count);
        state.accounting.claim_writeback(count);
    }

    fn rollback_writeback_claim(state: &mut PoolState, frames: &[PhysicalFrameKey], count: u32) {
        for frame in frames {
            if let Some(entry) = state.frames.get_mut(&frame.coordinate) {
                entry.writeback_claimed = false;
            }
        }
        state
            .accounting
            .release_speculative(crate::PhysicalSpeculativeWorkKind::WriteBehind, count);
        state.accounting.release_writeback(count);
    }

    pub(crate) fn complete_writeback_claim(
        &self,
        frames: &[PhysicalFrameKey],
    ) -> Result<(), PhysicalResidencyDenial> {
        let mut state = self.lock();
        if !state.accepting {
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        for frame in frames {
            let Some(entry) = state.frames.get(&frame.coordinate) else {
                return Err(Self::deny(
                    &mut state,
                    PhysicalResidencyDenial::WriteBackFrameNotDirty,
                ));
            };
            if !entry.dirty || !entry.writeback_claimed {
                return Err(Self::deny(
                    &mut state,
                    PhysicalResidencyDenial::WriteBackFrameNotDirty,
                ));
            }
        }
        let mut published_candidates = 0_u64;
        for frame in frames {
            let entry = state
                .frames
                .get_mut(&frame.coordinate)
                .expect("validated writeback frame remains resident");
            entry.dirty = false;
            entry.dirty_generation = None;
            entry.writeback_claimed = false;
            entry.invalidate_integrity_validation();
            if entry.origin.is_candidate() {
                entry.origin = FrameOrigin::Fault;
                published_candidates += 1;
            }
        }
        let count = frames.len() as u32;
        for _ in frames {
            let candidate_removed = published_candidates > 0;
            state.accounting.mark_clean(candidate_removed, true);
            published_candidates = published_candidates.saturating_sub(1);
        }
        state.accounting.release_writeback(count);
        for frame in frames {
            let becomes_evictable = state
                .frames
                .get(&frame.coordinate)
                .is_some_and(FrameEntry::is_evictable);
            if becomes_evictable {
                state.append_evictable(frame.coordinate);
            }
        }
        self.changed.notify_all();
        Ok(())
    }

    pub(crate) fn release_writeback_claim(&self, frames: &[PhysicalFrameKey]) {
        let mut state = self.lock();
        for frame in frames {
            if let Some(entry) = state.frames.get_mut(&frame.coordinate) {
                entry.writeback_claimed = false;
            }
        }
        let count = frames.len() as u32;
        state.accounting.release_writeback(count);
        self.changed.notify_all();
    }
}

struct PreparedWritebackClaim {
    frames: Box<[PhysicalFrameKey]>,
    resident_bytes: Vec<Arc<Vec<u8>>>,
    range_postures: Vec<crate::PhysicalWritebackRangePosture>,
}

pub(super) struct ClaimedWritebackFrames {
    pub(super) frames: Box<[PhysicalFrameKey]>,
    pub(super) resident_bytes: Vec<Arc<Vec<u8>>>,
    pub(super) range_postures: Vec<crate::PhysicalWritebackRangePosture>,
}
