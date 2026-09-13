use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_integrity::{InlinePageIntegrityValidation, PhysicalArtifactScope};

use super::super::admitted_artifact::IntegrityAdmittedRecoveryArtifact;
use super::super::families::page::IntegrityAdmittedPageFrame;
use super::super::{ObservedRecoverySource, RecoveryIntegrityIngressCounters};
use super::{recorded, rejected_integrity, RecoveryIntegrityIngressAttempt};

impl<'media> IntegrityAdmittedRecoveryArtifact<'media> {
    pub(crate) fn bind_page_frame(
        observed: &'media ObservedRecoveryArtifact,
        expected_scope: PhysicalArtifactScope,
        validation: InlinePageIntegrityValidation<'media>,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> RecoveryIntegrityIngressAttempt<'media> {
        match validation {
            InlinePageIntegrityValidation::Intact(validated) => recorded(
                expected_scope,
                IntegrityAdmittedPageFrame::bind(
                    ObservedRecoverySource::complete(observed, expected_scope),
                    validated,
                )
                .map(Self::PageFrame),
                counters,
            ),
            InlinePageIntegrityValidation::Rejected(rejection) => {
                rejected_integrity(expected_scope, rejection, counters)
            }
        }
    }
}
