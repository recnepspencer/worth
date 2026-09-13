use worth_store::physical_runtime::ObservedWalArtifact;
use worth_store_physical_integrity::{
    PhysicalArtifactScope, PhysicalByteRange, WalFrameIntegrityValidation,
};

use super::super::admitted_artifact::IntegrityAdmittedRecoveryArtifact;
use super::super::families::wal::IntegrityAdmittedWalFrame;
use super::super::{ObservedWalFrameSource, RecoveryIntegrityIngressCounters};
use super::{recorded, rejected_integrity, RecoveryIntegrityIngressAttempt};

impl<'media> IntegrityAdmittedRecoveryArtifact<'media> {
    pub(crate) fn bind_wal_frame(
        owner: &worth_store::physical_runtime::PhysicalRecoveryCoordination,
        observed: &'media ObservedWalArtifact,
        expected_scope: PhysicalArtifactScope,
        relative_range: PhysicalByteRange,
        validation: WalFrameIntegrityValidation<'media>,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> RecoveryIntegrityIngressAttempt<'media> {
        match validation {
            WalFrameIntegrityValidation::Intact(validated) => recorded(
                expected_scope,
                IntegrityAdmittedWalFrame::bind(
                    owner,
                    ObservedWalFrameSource::new(observed, expected_scope, relative_range),
                    validated,
                )
                .map(Self::WalFrame),
                counters,
            ),
            WalFrameIntegrityValidation::Rejected(rejection) => {
                rejected_integrity(expected_scope, rejection, counters)
            }
        }
    }
}
