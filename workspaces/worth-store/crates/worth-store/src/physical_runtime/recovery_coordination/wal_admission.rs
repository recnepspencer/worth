use worth_store_physical_integrity::{
    IntegrityValidatedWalFrame, PhysicalArtifactScope, PhysicalByteRange,
};

use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, IntegrityAdmittedRecoveryWalSegmentBuilder,
    ObservedWalArtifact, RecoveryWalIntegrityAdmissionDenial,
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
        IntegrityAdmittedRecoveryWalFrame::bind(
            self,
            observed,
            expected_scope,
            relative_range,
            validated,
        )
    }

    pub fn begin_recovery_wal_segment<'coordination, 'source>(
        &'coordination self,
        observed: &'source ObservedWalArtifact,
        identity: worth_store_wal::WalSegmentArtifactIdentity,
    ) -> Result<
        IntegrityAdmittedRecoveryWalSegmentBuilder<'coordination, 'source>,
        RecoveryWalIntegrityAdmissionDenial,
    > {
        if observed.store_identity() != self.store {
            return Err(RecoveryWalIntegrityAdmissionDenial::ScopeMismatch);
        }
        if !observed.matches_media_generation(self.media_generation) {
            return Err(RecoveryWalIntegrityAdmissionDenial::SourceIncarnationMismatch);
        }
        IntegrityAdmittedRecoveryWalSegmentBuilder::begin(self, observed, identity)
    }
}
