use super::*;

impl PoolInner {
    pub(super) fn invalidate_clean(
        &self,
        key: PhysicalFrameKey,
    ) -> Result<(), PhysicalResidencyDenial> {
        if let Err(reason) = self.validate_key(key) {
            return Err(self.record_denial(reason));
        }
        let mut state = self.lock();
        if !state.accepting {
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        let Some(entry) = state.frames.get(&key.coordinate) else {
            return Ok(());
        };
        if let Err(reason) = Self::validate_clean_invalidation(entry) {
            return Err(Self::deny(&mut state, reason));
        }
        state.detach_evictable(key.coordinate);
        let removed = state
            .frames
            .remove(&key.coordinate)
            .expect("validated clean frame remains present");
        state.accounting.remove_frame(removed.accounting_removal());
        Ok(())
    }

    fn validate_clean_invalidation(entry: &FrameEntry) -> Result<(), PhysicalResidencyDenial> {
        match &entry.state {
            FrameState::LoadFailed(terminal) => {
                Err(PhysicalResidencyDenial::FrameLoadTerminated(*terminal))
            }
            FrameState::Loading | FrameState::CandidateReserved => {
                Err(PhysicalResidencyDenial::FrameIdentityOccupied)
            }
            FrameState::Resident(_) if entry.pins != 0 => Err(PhysicalResidencyDenial::FramePinned),
            FrameState::Resident(_) if entry.dirty => Err(PhysicalResidencyDenial::FrameDirty),
            FrameState::Resident(_) => Ok(()),
        }
    }

    pub(super) fn promote_clean_identity(
        &self,
        source: PhysicalFrameKey,
        target: PhysicalFrameKey,
    ) -> Result<(), PhysicalResidencyDenial> {
        if let Err(reason) = self.validate_key(source) {
            return Err(self.record_denial(reason));
        }
        if let Err(reason) = self.validate_key(target) {
            return Err(self.record_denial(reason));
        }
        if source.coordinate.length() != target.coordinate.length() {
            return Err(self.record_denial(PhysicalResidencyDenial::FrameLengthMismatch));
        }
        if source == target {
            return Err(self.record_denial(PhysicalResidencyDenial::IdentityAlreadyCurrent));
        }
        let mut state = self.lock();
        if !state.accepting {
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        if let Err(reason) = Self::validate_promotion_source(&state, source, target) {
            return Err(Self::deny(&mut state, reason));
        }
        if let Err(reason) = Self::validate_promotion_target(&state, target) {
            return Err(Self::deny(&mut state, reason));
        }
        Self::apply_clean_identity_promotion(&mut state, source, target);
        Ok(())
    }

    fn validate_promotion_source(
        state: &PoolState,
        source: PhysicalFrameKey,
        target: PhysicalFrameKey,
    ) -> Result<(), PhysicalResidencyDenial> {
        let Some(source_entry) = state.frames.get(&source.coordinate) else {
            return Err(PhysicalResidencyDenial::FrameNotResident);
        };
        if source_entry.pins != 0 {
            return Err(PhysicalResidencyDenial::FramePinned);
        }
        if source_entry.dirty {
            return Err(PhysicalResidencyDenial::FrameDirty);
        }
        if !matches!(source_entry.state, FrameState::Resident(_)) {
            return Err(PhysicalResidencyDenial::FrameNotResident);
        }
        let source_is_complete =
            source_entry.artifact_posture == FrameArtifactPosture::CompleteResident;
        if source_is_complete && target.coordinate.offset() != 0 {
            return Err(PhysicalResidencyDenial::CompleteArtifactRequiresOffsetZero);
        }
        if source_is_complete
            && source.coordinate.artifact() != target.coordinate.artifact()
            && state
                .frames
                .contains_artifact_alias(target.coordinate.artifact())
        {
            return Err(PhysicalResidencyDenial::ArtifactIdentityOccupied);
        }
        Ok(())
    }

    fn validate_promotion_target(
        state: &PoolState,
        target: PhysicalFrameKey,
    ) -> Result<(), PhysicalResidencyDenial> {
        let Some(target_entry) = state.frames.get(&target.coordinate) else {
            return Ok(());
        };
        match &target_entry.state {
            FrameState::LoadFailed(terminal) => {
                Err(PhysicalResidencyDenial::FrameLoadTerminated(*terminal))
            }
            FrameState::Loading | FrameState::CandidateReserved => {
                Err(PhysicalResidencyDenial::FrameIdentityOccupied)
            }
            FrameState::Resident(_) => {
                if target_entry.pins != 0 {
                    return Err(PhysicalResidencyDenial::FramePinned);
                }
                if target_entry.dirty {
                    return Err(PhysicalResidencyDenial::FrameDirty);
                }
                Ok(())
            }
        }
    }

    fn apply_clean_identity_promotion(
        state: &mut PoolState,
        source: PhysicalFrameKey,
        target: PhysicalFrameKey,
    ) {
        state.detach_evictable(source.coordinate);
        if state.frames.contains_key(&target.coordinate) {
            state.detach_evictable(target.coordinate);
            let removed = state
                .frames
                .remove(&target.coordinate)
                .expect("validated target remains resident");
            state.accounting.remove_frame(removed.accounting_removal());
        }
        let mut entry = state
            .frames
            .remove(&source.coordinate)
            .expect("validated source remains resident");
        entry.invalidate_integrity_validation();
        state.frames.insert(target.coordinate, entry);
        state.append_evictable(target.coordinate);
        state.accounting.record_identity_transition();
    }
}
