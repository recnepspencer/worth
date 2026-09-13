use crate::*;

pub(super) fn inspect(
    input: UntrustedPhysicalArtifact<'_>,
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityScrubInspection,
    PhysicalIntegrityObservationCounters,
) {
    macro_rules! inspect {
        ($validate:ident, $result:ident) => {{
            let (result, counters) = $validate(input, scope);
            let inspection = match result {
                $result::Intact(value) => PhysicalIntegrityScrubInspection::new(
                    PhysicalIntegrityObservationOutcome::Intact(scope),
                )
                .with_selector_identity(value.selector_identity()),
                $result::Rejected(rejection) => PhysicalIntegrityScrubInspection::new(
                    PhysicalIntegrityObservationOutcome::Rejected(rejection),
                ),
            };
            (inspection, counters)
        }};
    }
    if scope.artifact_family() == worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CurrentRootSelector {
        inspect!(validate_current_root_selector, CurrentRootSelectorIntegrityValidation)
    } else {
        inspect!(validate_previous_root_selector, PreviousRootSelectorIntegrityValidation)
    }
}
