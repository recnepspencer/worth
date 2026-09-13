use super::super::*;

impl PoolInner {
    pub(crate) fn publish_clean(
        &self,
        key: PhysicalFrameKey,
    ) -> Result<(), PhysicalResidencyDenial> {
        let mut state = self.lock();
        if !state.accepting {
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        let Some(entry) = state.frames.get_mut(&key.coordinate) else {
            return Err(Self::deny(
                &mut state,
                PhysicalResidencyDenial::FrameNotDirty,
            ));
        };
        if !entry.dirty {
            return Err(Self::deny(
                &mut state,
                PhysicalResidencyDenial::FrameNotDirty,
            ));
        }
        entry.dirty = false;
        entry.invalidate_integrity_validation();
        let was_candidate = entry.origin.is_candidate();
        if was_candidate {
            entry.origin = FrameOrigin::Fault;
        }
        if entry.artifact_posture == FrameArtifactPosture::CompleteCandidate {
            entry.artifact_posture = FrameArtifactPosture::CompleteResident;
        }
        state.accounting.mark_clean(was_candidate, false);
        Ok(())
    }
}
