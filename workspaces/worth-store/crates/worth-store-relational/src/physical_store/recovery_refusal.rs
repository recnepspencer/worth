//! Preserve unsupported-root schema meaning from physical integrity ingress.
use super::*;
use worth_store_physical_integrity::{PhysicalIntegrityRejection, PhysicalIntegrityVersionAxis};
use worth_store_recovery_runtime::{
    PhysicalRecoveryIntegrityObservationOutcome as Outcome,
    PhysicalRecoveryIntegrityRejection as Rejection,
};
pub(super) fn classify(outcome: PhysicalRecoveryOutcome) -> Refusal {
    let mut refusal = Refusal::new(Stage::Recovery, format!("{outcome:?}"));
    let observations = match &outcome {
        PhysicalRecoveryOutcome::Blocked(block) => block.evidence().integrity_observations(),
        PhysicalRecoveryOutcome::Refused(refused) => refused.integrity_observations(),
        _ => &[],
    };
    for observation in observations {
        if let Outcome::Rejected(Rejection::Integrity(PhysicalIntegrityRejection::Unsupported(
            version,
        ))) = observation.outcome()
        {
            if version.axis() == PhysicalIntegrityVersionAxis::EnvelopeSchema
                && version.scope().artifact_family() == worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootManifest
            {
                refusal.kind = RelationalStorePhysicalRefusalKind::UnsupportedRootSchema {
                    observed: version.observed(),
                };
                break;
            }
        }
    }
    refusal
}
