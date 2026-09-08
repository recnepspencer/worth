use crate::{
    validate_bootstrap_catalog, BootstrapCatalogIntegrityValidation, PhysicalArtifactScope,
    PhysicalIntegrityObservationCounters, PhysicalIntegrityObservationOutcome,
    PhysicalIntegrityScrubInspection, UntrustedPhysicalArtifact,
};

pub(super) fn inspect(
    input: UntrustedPhysicalArtifact<'_>,
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityScrubInspection,
    PhysicalIntegrityObservationCounters,
) {
    let (validation, counters) = validate_bootstrap_catalog(input, scope);
    let rejection = match validation {
        BootstrapCatalogIntegrityValidation::Intact(_) => {
            return (
                PhysicalIntegrityScrubInspection::new(PhysicalIntegrityObservationOutcome::Intact(
                    scope,
                )),
                counters,
            );
        }
        BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch) => mismatch.rejection(),
        BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported) => {
            unsupported.rejection()
        }
        BootstrapCatalogIntegrityValidation::Rejected(rejection) => rejection,
    };
    (
        PhysicalIntegrityScrubInspection::new(PhysicalIntegrityObservationOutcome::Rejected(
            rejection,
        )),
        counters,
    )
}
