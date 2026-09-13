use super::*;

impl PoolInner {
    pub(crate) fn reserve_dirty_replacement(
        &self,
        scope: PhysicalOperationAllocationScope,
        key: PhysicalFrameKey,
        expected: &Arc<Vec<u8>>,
        bytes: u64,
    ) -> Result<(), PhysicalResidencyDenial> {
        if let Err(reason) = self.validate_key(key) {
            return Err(self.record_denial(reason));
        }
        let mut state = self.lock();
        validate_transition(self, &mut state, scope, key, expected, bytes)?;
        state.accounting.reserve_dirty_replacement(scope, bytes);
        Ok(())
    }

    pub(crate) fn finish_dirty_replacement(
        &self,
        scope: PhysicalOperationAllocationScope,
        key: PhysicalFrameKey,
        expected: &Arc<Vec<u8>>,
        replacement: Arc<Vec<u8>>,
    ) -> Result<PhysicalResidentFrameGeneration, PhysicalResidencyDenial> {
        let mut state = self.lock();
        validate_clean_frame(&mut state, key, expected)?;
        let generation = state
            .advance_dirty_generation()
            .map_err(|denial| Self::deny(&mut state, denial))?;
        let resident_generation = state
            .advance_resident_generation()
            .map_err(|denial| Self::deny(&mut state, denial))?;
        let entry = state
            .frames
            .get_mut(&key.coordinate)
            .expect("validated clean frame remains resident");
        let was_candidate = entry.origin.is_candidate();
        entry.state = FrameState::Resident(replacement);
        entry.origin = FrameOrigin::DirtyReplacement;
        entry.allocation_scope = scope;
        entry.dirty = true;
        entry.dirty_generation = Some(generation);
        entry.resident_generation = Some(resident_generation);
        entry.invalidate_integrity_validation();
        state.accounting.mark_dirty(!was_candidate);
        Ok(resident_generation)
    }

    pub(crate) fn release_dirty_replacement(
        &self,
        scope: PhysicalOperationAllocationScope,
        bytes: u64,
    ) {
        let mut state = self.lock();
        state.accounting.release_dirty_replacement(scope, bytes);
        self.changed.notify_all();
    }

    pub(crate) fn dirty_replacement_allocator_failed(
        &self,
        scope: PhysicalOperationAllocationScope,
        bytes: u64,
    ) {
        let mut state = self.lock();
        state
            .accounting
            .dirty_replacement_allocator_failed(scope, bytes);
        self.changed.notify_all();
    }
}

fn validate_transition(
    owner: &PoolInner,
    state: &mut PoolState,
    scope: PhysicalOperationAllocationScope,
    key: PhysicalFrameKey,
    expected: &Arc<Vec<u8>>,
    bytes: u64,
) -> Result<(), PhysicalResidencyDenial> {
    if !state.accepting {
        return Err(PoolInner::deny(state, PhysicalResidencyDenial::PoolClosed));
    }
    if state.accounting.dirty_frames() >= owner.limits.dirty_frames() {
        let current = u64::from(state.accounting.dirty_frames());
        return Err(owner.pressure(
            state,
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::DirtyFrames,
                scope,
                requested: 1,
                current,
                limit: u64::from(owner.limits.dirty_frames()),
            },
        ));
    }
    let replacement_current = state.accounting.dirty_replacement_bytes();
    if replacement_current.saturating_add(bytes) > owner.limits.dirty_replacement_bytes() {
        return Err(owner.pressure(
            state,
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::DirtyReplacementBytes,
                scope,
                requested: bytes,
                current: replacement_current,
                limit: owner.limits.dirty_replacement_bytes(),
            },
        ));
    }
    let total_current = owner.current_admitted_bytes(state);
    if total_current.saturating_add(bytes) > owner.limits.total_bytes() {
        return Err(owner.pressure(
            state,
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::TotalBytes,
                scope,
                requested: bytes,
                current: total_current,
                limit: owner.limits.total_bytes(),
            },
        ));
    }
    validate_clean_frame(state, key, expected)
}

fn validate_clean_frame(
    state: &mut PoolState,
    key: PhysicalFrameKey,
    expected: &Arc<Vec<u8>>,
) -> Result<(), PhysicalResidencyDenial> {
    if !state.accepting {
        return Err(PoolInner::deny(state, PhysicalResidencyDenial::PoolClosed));
    }
    let Some(entry) = state.frames.get(&key.coordinate) else {
        return Err(PoolInner::deny(
            state,
            PhysicalResidencyDenial::FrameNotResident,
        ));
    };
    if entry.pins != 1 {
        return Err(PoolInner::deny(state, PhysicalResidencyDenial::FramePinned));
    }
    if entry.dirty {
        return Err(PoolInner::deny(state, PhysicalResidencyDenial::FrameDirty));
    }
    if entry.writeback_claimed {
        return Err(PoolInner::deny(
            state,
            PhysicalResidencyDenial::WriteBackFrameAlreadyClaimed,
        ));
    }
    match &entry.state {
        FrameState::Resident(current) if Arc::ptr_eq(current, expected) => Ok(()),
        _ => Err(PoolInner::deny(
            state,
            PhysicalResidencyDenial::FrameNotResident,
        )),
    }
}
