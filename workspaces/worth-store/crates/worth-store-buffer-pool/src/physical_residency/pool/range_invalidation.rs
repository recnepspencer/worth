use super::*;

impl PhysicalResidencyPool {
    /// Evicts only clean, unpinned frames overlapping an artifact byte range.
    /// Every matching frame is validated before any eviction; neighboring
    /// frames and their integrity generations remain untouched.
    pub fn invalidate_clean_range(
        &self,
        artifact: RecordArtifactFile,
        offset: u64,
        length: u64,
    ) -> Result<(), PhysicalResidencyDenial> {
        let Some(end) = offset.checked_add(length).filter(|end| *end > offset) else {
            return Err(self
                .inner
                .record_denial(PhysicalResidencyDenial::FrameLengthMismatch));
        };
        let matches = |coordinate: RecordFrameCoordinate| {
            coordinate.artifact() == artifact
                && coordinate.offset() < end
                && coordinate
                    .offset()
                    .saturating_add(u64::from(coordinate.length()))
                    > offset
        };
        let mut state = self.inner.lock();
        if !state.accepting {
            return Err(PoolInner::deny(
                &mut state,
                PhysicalResidencyDenial::PoolClosed,
            ));
        }
        if state.frames.contains_artifact_alias(artifact) {
            return Err(PoolInner::deny(
                &mut state,
                PhysicalResidencyDenial::ArtifactIdentityOccupied,
            ));
        }
        for slot in 0..state.frames.slot_count() {
            let Some(coordinate) = state
                .frames
                .exact_coordinate_at(slot)
                .filter(|coordinate| matches(*coordinate))
            else {
                continue;
            };
            if let Err(reason) = PoolInner::validate_clean_invalidation(
                state.frames.get(&coordinate).expect("occupied slot"),
            ) {
                return Err(PoolInner::deny(&mut state, reason));
            }
        }
        for slot in 0..state.frames.slot_count() {
            let Some(coordinate) = state
                .frames
                .exact_coordinate_at(slot)
                .filter(|coordinate| matches(*coordinate))
            else {
                continue;
            };
            state.detach_evictable(coordinate);
            let removed = state
                .frames
                .remove(&coordinate)
                .expect("validated range frame");
            state.accounting.remove_frame(removed.accounting_removal());
        }
        Ok(())
    }
}
