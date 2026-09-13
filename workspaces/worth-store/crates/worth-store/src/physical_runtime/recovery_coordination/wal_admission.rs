use worth_store_physical_integrity::{
    IntegrityValidatedWalFrame, PhysicalArtifactScope, PhysicalByteRange,
};

use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, IntegrityAdmittedRecoveryWalSegment, ObservedWalArtifact,
    RecoveryWalIntegrityAdmissionDenial,
};

impl super::PhysicalRecoveryCoordination {
    pub fn admit_recovery_wal_frame(
        &self,
        observed: &ObservedWalArtifact,
        expected_scope: PhysicalArtifactScope,
        relative_range: PhysicalByteRange,
        validated: IntegrityValidatedWalFrame<'_>,
    ) -> Result<IntegrityAdmittedRecoveryWalFrame, RecoveryWalIntegrityAdmissionDenial> {
        if expected_scope.store_identity() != self.store {
            return Err(RecoveryWalIntegrityAdmissionDenial::ScopeMismatch);
        }
        if !observed.matches_media_generation(self.media_generation) {
            return Err(RecoveryWalIntegrityAdmissionDenial::SourceIncarnationMismatch);
        }
        IntegrityAdmittedRecoveryWalFrame::bind(observed, expected_scope, relative_range, validated)
    }

    pub fn retain_admitted_recovery_wal_segment(
        &self,
        observed: &ObservedWalArtifact,
        identity: worth_store_wal::WalSegmentArtifactIdentity,
        frames: Vec<IntegrityAdmittedRecoveryWalFrame>,
    ) -> Option<IntegrityAdmittedRecoveryWalSegment> {
        (observed.store_identity() == self.store
            && observed.matches_media_generation(self.media_generation))
        .then_some(())?;
        frames
            .iter()
            .all(|frame| frame.scope().store_identity() == self.store)
            .then_some(())?;
        IntegrityAdmittedRecoveryWalSegment::from_complete_frames(observed, identity, frames)
    }
}
