use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_integrity::{
    PhysicalArtifactScope, SegmentMembershipBlockIntegrityValidation,
};

use super::super::admitted_artifact::IntegrityAdmittedRecoveryArtifact;
use super::super::families::segment_membership::IntegrityAdmittedSegmentMembershipBlock;
use super::super::{ObservedRecoverySource, RecoveryIntegrityIngressCounters};
use super::{recorded, rejected_integrity, RecoveryIntegrityIngressAttempt};

impl<'media> IntegrityAdmittedRecoveryArtifact<'media> {
    pub(crate) fn bind_segment_membership_block(
        observed: &'media ObservedRecoveryArtifact,
        expected_scope: PhysicalArtifactScope,
        validation: SegmentMembershipBlockIntegrityValidation<'media>,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> RecoveryIntegrityIngressAttempt<'media> {
        match validation {
            SegmentMembershipBlockIntegrityValidation::Intact(validated) => recorded(
                expected_scope,
                IntegrityAdmittedSegmentMembershipBlock::bind(
                    ObservedRecoverySource::complete(observed, expected_scope),
                    validated,
                )
                .map(Self::SegmentMembershipBlock),
                counters,
            ),
            SegmentMembershipBlockIntegrityValidation::Rejected(rejection) => {
                rejected_integrity(expected_scope, rejection, counters)
            }
        }
    }
}
