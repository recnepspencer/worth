use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_integrity::{
    FreeSpaceHeaderIntegrityValidation, FreeSpaceMembershipBlockIntegrityValidation,
    PhysicalArtifactScope,
};

use super::super::admitted_artifact::IntegrityAdmittedRecoveryArtifact;
use super::super::families::free_space::{
    IntegrityAdmittedFreeSpaceHeader, IntegrityAdmittedFreeSpaceMembershipBlock,
};
use super::super::{ObservedRecoverySource, RecoveryIntegrityIngressCounters};
use super::{recorded, rejected_integrity, RecoveryIntegrityIngressAttempt};

macro_rules! free_space_source_binding {
    ($name:ident, $validation:ident, $wrapper:ty, $variant:ident) => {
        pub(crate) fn $name(
            observed: &'media ObservedRecoveryArtifact,
            expected_scope: PhysicalArtifactScope,
            validation: $validation<'media>,
            counters: &mut RecoveryIntegrityIngressCounters,
        ) -> RecoveryIntegrityIngressAttempt<'media> {
            match validation {
                $validation::Intact(validated) => recorded(
                    expected_scope,
                    <$wrapper>::bind(
                        ObservedRecoverySource::complete(observed, expected_scope),
                        validated,
                    )
                    .map(Self::$variant),
                    counters,
                ),
                $validation::Rejected(rejection) => {
                    rejected_integrity(expected_scope, rejection, counters)
                }
            }
        }
    };
}

impl<'media> IntegrityAdmittedRecoveryArtifact<'media> {
    free_space_source_binding!(
        bind_free_space_header,
        FreeSpaceHeaderIntegrityValidation,
        IntegrityAdmittedFreeSpaceHeader<'media>,
        FreeSpaceHeader
    );
    free_space_source_binding!(
        bind_free_space_membership_block,
        FreeSpaceMembershipBlockIntegrityValidation,
        IntegrityAdmittedFreeSpaceMembershipBlock<'media>,
        FreeSpaceMembershipBlock
    );
}
