use super::*;

mod declaration;

pub(super) use declaration::candidate_batch_operation_bytes;

impl PoolInner {
    pub(crate) fn validate_candidate_projection_start(
        &self,
    ) -> Result<(), PhysicalResidencyDenial> {
        let mut state = self.lock();
        if !state.accepting {
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        Ok(())
    }

    pub(crate) fn admit_candidate_batch<'grant>(
        self: &Arc<Self>,
        admission: PhysicalCandidateBatchAdmission<'grant>,
        keys: &[PhysicalCandidateFrameKey],
    ) -> Result<PhysicalCandidateBatchReservation<'grant>, PhysicalResidencyDenial> {
        if admission.candidate_count.get() != keys.len() {
            return Err(self.record_denial(
                PhysicalResidencyDenial::CandidateCardinalityMismatch {
                    declared: admission.candidate_count.get(),
                    provided: keys.len(),
                },
            ));
        }
        let admitted = self.validate_candidate_set(keys)?;
        let mut state = self.lock();
        let scope = admission.scope();
        self.admit_candidate_set(&mut state, scope, keys)?;
        Ok(PhysicalCandidateBatchReservation {
            owner: Arc::clone(self),
            keys: admitted,
            allocation_use: admission.allocation_use,
            armed: true,
        })
    }

    fn admit_candidate_set(
        &self,
        state: &mut PoolState,
        scope: PhysicalOperationAllocationScope,
        keys: &[PhysicalCandidateFrameKey],
    ) -> Result<(), PhysicalResidencyDenial> {
        if !state.accepting {
            return Err(Self::deny(state, PhysicalResidencyDenial::PoolClosed));
        }
        if state.active_candidate_publications >= self.limits.pin_leases() {
            return Err(Self::deny(
                state,
                PhysicalResidencyDenial::CandidatePublicationActive,
            ));
        }
        for candidate in keys {
            if let Err(reason) =
                Self::validate_candidate_identity_available(state, candidate.frame_key())
            {
                return Err(Self::deny(state, reason));
            }
        }
        self.validate_candidate_capacity(state, scope, keys)?;
        state.active_candidate_publications += 1;
        Ok(())
    }

    fn validate_candidate_identity_available(
        state: &PoolState,
        key: PhysicalFrameKey,
    ) -> Result<(), PhysicalResidencyDenial> {
        if let Some(entry) = state.frames.get(&key.coordinate) {
            return match &entry.state {
                FrameState::LoadFailed(terminal) => {
                    Err(PhysicalResidencyDenial::FrameLoadTerminated(*terminal))
                }
                FrameState::Loading | FrameState::CandidateReserved => {
                    Err(PhysicalResidencyDenial::FrameIdentityOccupied)
                }
                FrameState::Resident(_) => Err(PhysicalResidencyDenial::FrameAlreadyResident),
            };
        }
        if state
            .frames
            .contains_artifact_alias(key.coordinate.artifact())
        {
            return Err(PhysicalResidencyDenial::ArtifactIdentityOccupied);
        }
        Ok(())
    }

    fn validate_candidate_capacity(
        &self,
        state: &mut PoolState,
        scope: PhysicalOperationAllocationScope,
        keys: &[PhysicalCandidateFrameKey],
    ) -> Result<(), PhysicalResidencyDenial> {
        let max_frame = keys
            .iter()
            .map(|candidate| u64::from(candidate.frame_key().coordinate.length()))
            .max()
            .expect("nonempty candidate set");
        self.validate_candidate_replacement_window(state, scope, max_frame)?;
        self.validate_candidate_live_ceilings(state, scope)
    }

    fn validate_candidate_replacement_window(
        &self,
        state: &mut PoolState,
        scope: PhysicalOperationAllocationScope,
        max_frame: u64,
    ) -> Result<(), PhysicalResidencyDenial> {
        let (evictable_bytes, evictable_frames) = state
            .frames
            .values()
            .filter(|entry| entry.is_evictable())
            .fold((0_u64, 0_usize), |(bytes, frames), entry| {
                (bytes.saturating_add(entry.bytes), frames.saturating_add(1))
            });
        let fixed_bytes = state
            .accounting
            .resident_bytes()
            .saturating_sub(evictable_bytes);
        let fixed_frames = usize::try_from(state.accounting.frame_entries())
            .expect("admitted frame count fits usize")
            .saturating_sub(evictable_frames);
        if fixed_bytes.saturating_add(max_frame) > self.limits.resident_bytes() {
            return Err(self.pressure(
                state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::ResidentBytes,
                    scope,
                    requested: max_frame,
                    current: fixed_bytes,
                    limit: self.limits.resident_bytes(),
                },
            ));
        }
        if fixed_frames.saturating_add(1) > self.limits.frame_entries() as usize {
            return Err(self.pressure(
                state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::FrameEntries,
                    scope,
                    requested: 1,
                    current: fixed_frames as u64,
                    limit: u64::from(self.limits.frame_entries()),
                },
            ));
        }
        Ok(())
    }

    fn validate_candidate_live_ceilings(
        &self,
        state: &mut PoolState,
        scope: PhysicalOperationAllocationScope,
    ) -> Result<(), PhysicalResidencyDenial> {
        if state.accounting.dirty_frames() >= self.limits.dirty_frames() {
            return Err(self.pressure(
                state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::DirtyFrames,
                    scope,
                    requested: 1,
                    current: u64::from(state.accounting.dirty_frames()),
                    limit: u64::from(self.limits.dirty_frames()),
                },
            ));
        }
        if state.accounting.pinned_frames() >= self.limits.pinned_frames() {
            return Err(self.pressure(
                state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::PinnedFrames,
                    scope,
                    requested: 1,
                    current: u64::from(state.accounting.pinned_frames()),
                    limit: u64::from(self.limits.pinned_frames()),
                },
            ));
        }
        if state.accounting.pin_leases() >= self.limits.pin_leases() {
            return Err(self.pressure(
                state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::PinLeases,
                    scope,
                    requested: 1,
                    current: u64::from(state.accounting.pin_leases()),
                    limit: u64::from(self.limits.pin_leases()),
                },
            ));
        }
        Ok(())
    }

    pub(crate) fn reserve_next_candidate(
        &self,
        scope: PhysicalOperationAllocationScope,
        candidate: PhysicalCandidateFrameKey,
    ) -> Result<(), PhysicalResidencyDenial> {
        let key = candidate.frame_key();
        let mut state = self.lock();
        if let Err(reason) = Self::validate_candidate_identity_available(&state, key) {
            return Err(Self::deny(&mut state, reason));
        }
        if state.accounting.dirty_frames() >= self.limits.dirty_frames() {
            let current = u64::from(state.accounting.dirty_frames());
            return Err(self.pressure(
                &mut state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::DirtyFrames,
                    scope,
                    requested: 1,
                    current,
                    limit: u64::from(self.limits.dirty_frames()),
                },
            ));
        }
        self.reserve_frame_space(&mut state, scope, u64::from(key.coordinate.length()))?;
        state.frames.insert(
            key.coordinate,
            FrameEntry {
                state: FrameState::CandidateReserved,
                origin: FrameOrigin::Candidate,
                allocation_scope: scope,
                pins: 1,
                dirty: true,
                dirty_generation: None,
                writeback_claimed: false,
                bytes: u64::from(key.coordinate.length()),
                older_evictable: None,
                newer_evictable: None,
                loading_identity: None,
                loading_waiters: 0,
                artifact_posture: if candidate.is_complete_artifact() {
                    FrameArtifactPosture::CompleteCandidate
                } else {
                    FrameArtifactPosture::Fragment
                },
                resident_generation: None,
                integrity_validation: None,
            },
        );
        state
            .accounting
            .admit_frame(scope, u64::from(key.coordinate.length()), true, true);
        state.loading_frames += 1;
        Ok(())
    }

    pub(crate) fn finish_candidate_batch(&self) {
        let mut state = self.lock();
        state.active_candidate_publications = state
            .active_candidate_publications
            .checked_sub(1)
            .expect("candidate batch finished without an active publication");
        self.changed.notify_all();
    }

    pub(crate) fn finish_candidate(
        self: &Arc<Self>,
        key: PhysicalFrameKey,
        bytes: Arc<Vec<u8>>,
    ) -> Result<DirtyPhysicalFrame, PhysicalResidencyDenial> {
        let mut state = self.lock();
        if !state.accepting {
            cancel_candidate_locked(&mut state, key.coordinate);
            self.changed.notify_all();
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        let Some(entry) = state.frames.get(&key.coordinate) else {
            return Err(Self::deny(
                &mut state,
                PhysicalResidencyDenial::FrameNotResident,
            ));
        };
        if !matches!(entry.state, FrameState::CandidateReserved) {
            return Err(Self::deny(
                &mut state,
                PhysicalResidencyDenial::FrameAlreadyResident,
            ));
        }
        let generation = state
            .advance_dirty_generation()
            .map_err(|denial| Self::deny(&mut state, denial))?;
        let resident_generation = state
            .advance_resident_generation()
            .map_err(|denial| Self::deny(&mut state, denial))?;
        let entry = state
            .frames
            .get_mut(&key.coordinate)
            .expect("validated candidate remains reserved");
        entry.state = FrameState::Resident(Arc::clone(&bytes));
        entry.dirty_generation = Some(generation);
        entry.resident_generation = Some(resident_generation);
        entry.invalidate_integrity_validation();
        state.loading_frames -= 1;
        state.accounting.finish_loading();
        self.changed.notify_all();
        Ok(DirtyPhysicalFrame {
            lease: Some(PhysicalFrameLease {
                owner: Arc::clone(self),
                key,
                bytes,
                resident_generation,
            }),
        })
    }

    pub(crate) fn cancel_candidate(&self, key: PhysicalFrameKey) {
        let mut state = self.lock();
        cancel_candidate_locked(&mut state, key.coordinate);
        self.changed.notify_all();
    }

    pub(crate) fn candidate_allocator_failed(&self, key: PhysicalFrameKey) {
        let mut state = self.lock();
        if matches!(
            state.frames.get(&key.coordinate).map(|entry| &entry.state),
            Some(FrameState::CandidateReserved)
        ) {
            let entry = state
                .frames
                .remove(&key.coordinate)
                .expect("candidate reservation remains present");
            state.accounting.candidate_allocator_failed(
                entry.allocation_scope,
                entry.bytes,
                entry.pins,
            );
            state.loading_frames -= 1;
            state.accounting.finish_loading();
        }
        self.changed.notify_all();
    }

    pub(crate) fn discard_dirty_candidate(
        &self,
        key: PhysicalFrameKey,
    ) -> Result<(), PhysicalResidencyDenial> {
        let mut state = self.lock();
        let Some(entry) = state.frames.get(&key.coordinate) else {
            return Err(Self::deny(
                &mut state,
                PhysicalResidencyDenial::FrameNotResident,
            ));
        };
        if !entry.dirty {
            return Err(Self::deny(
                &mut state,
                PhysicalResidencyDenial::FrameNotDirty,
            ));
        }
        if !entry.origin.is_candidate()
            || entry.pins != 1
            || entry.writeback_claimed
            || !matches!(entry.state, FrameState::Resident(_))
        {
            return Err(Self::deny(
                &mut state,
                PhysicalResidencyDenial::CandidatePublicationActive,
            ));
        }
        let removed = state
            .frames
            .remove(&key.coordinate)
            .expect("validated candidate frame remains resident");
        state.accounting.remove_frame(removed.accounting_removal());
        state.accounting.record_administrative_drain();
        self.changed.notify_all();
        Ok(())
    }
}

fn cancel_candidate_locked(state: &mut PoolState, coordinate: RecordFrameCoordinate) {
    if matches!(
        state.frames.get(&coordinate).map(|entry| &entry.state),
        Some(FrameState::CandidateReserved)
    ) {
        let entry = state
            .frames
            .remove(&coordinate)
            .expect("candidate reservation remains present");
        state.accounting.remove_frame(entry.accounting_removal());
        state.loading_frames -= 1;
        state.accounting.finish_loading();
    }
}
