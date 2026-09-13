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

    pub(crate) fn selected_frames<'a>(
        &'a self,
        selected: &'a worth_store_recovery_physics::SelectedPhysicalWalTail,
    ) -> impl Iterator<Item = &'a IntegrityAdmittedRecoveryWalFrame> {
        selected.segments().iter().flat_map(|selected_segment| {
            let admitted = self.segment(selected_segment.identity());
            selected_segment.frame_facts().iter().map(|selected_frame| {
                admitted
                    .frames()
                    .iter()
                    .find(|frame| frame.lsn_range() == selected_frame.lsn_range())
                    .expect("selected WAL frame remains bound to its C.9 admission")
            })
        })
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
