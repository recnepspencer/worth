use worth_store::physical_runtime::{
    ObservedWalArtifact, RecoveryWalAllocationDenial, RecoveryWalIntegrityAdmissionDenial,
};
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
    ) -> Result<RecoveryIntegrityIngressAttempt<'media>, RecoveryWalAllocationDenial> {
        match validation {
            WalFrameIntegrityValidation::Intact(validated) => {
                let binding = IntegrityAdmittedWalFrame::bind(
                    owner,
                    ObservedWalFrameSource::new(observed, expected_scope, relative_range),
                    validated,
                );
                let outcome = match binding {
                    Ok(frame) => Ok(Self::WalFrame(frame)),
                    Err(RecoveryWalIntegrityAdmissionDenial::Allocation(cause)) => {
                        return Err(cause)
                    }
                    Err(denial) => Err(super::super::families::wal::map_store_denial(denial)),
                };
                Ok(recorded(expected_scope, outcome, counters))
            }
            WalFrameIntegrityValidation::Rejected(rejection) => {
                Ok(rejected_integrity(expected_scope, rejection, counters))
            }
        }
    }
}
