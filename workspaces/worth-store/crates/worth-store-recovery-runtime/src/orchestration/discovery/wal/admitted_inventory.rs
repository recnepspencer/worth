use worth_store::physical_runtime::{
    recovery_wal::WalSegmentArtifactIdentity, IntegrityAdmittedRecoveryWalFrame,
    IntegrityAdmittedRecoveryWalSegment,
};

#[derive(Default)]
pub(crate) struct AdmittedWalInventory {
    segments: Vec<IntegrityAdmittedRecoveryWalSegment>,
}

impl AdmittedWalInventory {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        self.segments.iter().try_fold(
            u64::try_from(self.segments.capacity())
                .ok()?
                .checked_mul(std::mem::size_of::<IntegrityAdmittedRecoveryWalSegment>() as u64)?,
            |bytes, segment| bytes.checked_add(segment.owned_heap_bytes()?),
        )
    }

    pub(super) fn push(&mut self, segment: IntegrityAdmittedRecoveryWalSegment) {
        self.segments.push(segment);
    }

    /// Every admitted frame in the retained tail or a checkpoint-covered segment.
    ///
    /// The retained tail's selected frames omit the prefix the checkpoint
    /// already covers. Retirement intents in that prefix are still obligations.
    pub(crate) fn recoverable_frames<'a>(
        &'a self,
        selected: &'a worth_store_recovery_physics::SelectedPhysicalWalTail,
    ) -> Vec<&'a IntegrityAdmittedRecoveryWalFrame> {
        self.recoverable_frame_iter(selected).collect()
    }

    pub(crate) fn recoverable_frame_iter<'a>(
        &'a self,
        selected: &'a worth_store_recovery_physics::SelectedPhysicalWalTail,
    ) -> impl Iterator<Item = &'a IntegrityAdmittedRecoveryWalFrame> + 'a {
        self.segments
            .iter()
            .filter(move |segment| {
                let identity = segment.inspection().identity();
                selected
                    .segments()
                    .iter()
                    .any(|candidate| candidate.identity() == identity)
                    || selected
                        .checkpoint_covered()
                        .iter()
                        .any(|covered| covered.identity() == identity)
            })
            .flat_map(|segment| segment.frames().iter())
    }

    pub(crate) fn cleanup_segments(
        &self,
        identities: impl IntoIterator<Item = WalSegmentArtifactIdentity>,
    ) -> Vec<IntegrityAdmittedRecoveryWalSegment> {
        identities
            .into_iter()
            .map(|identity| self.segment(identity).clone())
            .collect()
    }

    fn segment(
        &self,
        identity: WalSegmentArtifactIdentity,
    ) -> &IntegrityAdmittedRecoveryWalSegment {
        self.segments
            .iter()
            .find(|segment| segment.inspection().identity() == identity)
            .expect("C.8 selection consumes a C.9-admitted WAL segment")
    }
}
