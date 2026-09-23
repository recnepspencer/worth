use worth_store::physical_runtime::{
    recovery_wal::WalSegmentArtifactIdentity, IntegrityAdmittedRecoveryWalFrame,
    IntegrityAdmittedRecoveryWalSegment,
};

#[derive(Default)]
pub(crate) struct AdmittedWalInventory {
    segments: Vec<IntegrityAdmittedRecoveryWalSegment>,
}

impl AdmittedWalInventory {
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
        let mut frames = Vec::new();
        for segment in &self.segments {
            let identity = segment.inspection().identity();
            let named = selected
                .segments()
                .iter()
                .any(|candidate| candidate.identity() == identity)
                || selected
                    .checkpoint_covered()
                    .iter()
                    .any(|covered| covered.identity() == identity);
            if named {
                frames.extend(segment.frames());
            }
        }
        frames
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
