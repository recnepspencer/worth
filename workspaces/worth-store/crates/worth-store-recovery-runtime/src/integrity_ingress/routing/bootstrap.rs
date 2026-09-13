use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_integrity::{BootstrapCatalogIntegrityValidation, PhysicalArtifactScope};

use super::super::admitted_artifact::IntegrityAdmittedRecoveryArtifact;
use super::super::families::bootstrap::IntegrityAdmittedBootstrapCatalog;
use super::super::{ObservedRecoverySource, RecoveryIntegrityIngressCounters};
use super::{recorded, rejected_integrity, RecoveryIntegrityIngressAttempt};

impl<'media> IntegrityAdmittedRecoveryArtifact<'media> {
    pub(crate) fn bind_bootstrap_catalog(
        observed: &'media ObservedRecoveryArtifact,
        expected_scope: PhysicalArtifactScope,
        validation: BootstrapCatalogIntegrityValidation<'media>,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> RecoveryIntegrityIngressAttempt<'media> {
        match validation {
            BootstrapCatalogIntegrityValidation::Intact(validated) => recorded(
                expected_scope,
                IntegrityAdmittedBootstrapCatalog::bind(
                    ObservedRecoverySource::complete(observed, expected_scope),
                    validated,
                )
                .map(Self::BootstrapCatalog),
                counters,
            ),
            BootstrapCatalogIntegrityValidation::Rejected(rejection) => {
                rejected_integrity(expected_scope, rejection, counters)
            }
            BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch) => {
                rejected_integrity(expected_scope, mismatch.rejection(), counters)
            }
            BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported) => {
                rejected_integrity(expected_scope, unsupported.rejection(), counters)
            }
        }
    }
}
