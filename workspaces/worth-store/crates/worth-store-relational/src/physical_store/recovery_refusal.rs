//! Preserve unsupported-root schema meaning from physical integrity ingress.
use super::*;
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_integrity::{PhysicalIntegrityRejection, PhysicalIntegrityVersionAxis};
use worth_store_recovery_runtime::{
    PhysicalRecoveryIntegrityObservationOutcome as Outcome,
    PhysicalRecoveryIntegrityRejection as Rejection, PhysicalRecoveryRootProtocolDenial,
    PhysicalRecoverySourceDenial,
};

pub(super) fn classify(outcome: PhysicalRecoveryOutcome) -> Refusal {
    let (sources, observations, effects) = match &outcome {
        PhysicalRecoveryOutcome::Blocked(block) => (
            block.evidence().source_denials.as_slice(),
            block.evidence().integrity_observations(),
            Some(block.recovery_effects()),
        ),
        PhysicalRecoveryOutcome::Refused(refused) => (
            refused.root_protocol_denials(),
            refused.integrity_observations(),
            Some(refused.recovery_effects()),
        ),
        PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => (
            failure.root_protocol_denials(),
            &[][..],
            Some(failure.recovery_effects()),
        ),
        PhysicalRecoveryOutcome::Recovered(_) => (&[][..], &[][..], None),
    };
    let schema = sources
        .iter()
        .find_map(|source| match source {
            PhysicalRecoverySourceDenial::RootProtocol {
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(rejection),
                ..
            } => unsupported_root(*rejection),
            _ => None,
        })
        .or_else(|| {
            observations
                .iter()
                .find_map(|observation| match observation.outcome() {
                    Outcome::Rejected(Rejection::Integrity(rejection)) => {
                        unsupported_root(rejection)
                    }
                    _ => None,
                })
        });
    let detail = schema.map_or_else(
        || format!("{outcome:?}"),
        |observed| format!("UnsupportedRootSchema {{ observed: {observed} }}"),
    );
    let mut refusal = Refusal::new(Stage::Recovery, detail);
    refusal.recovery_effects = effects;
    if let Some(observed) = schema {
        refusal.kind = RelationalStorePhysicalRefusalKind::UnsupportedRootSchema { observed };
    }
    refusal
}

fn unsupported_root(rejection: PhysicalIntegrityRejection) -> Option<u32> {
    let PhysicalIntegrityRejection::Unsupported(version) = rejection else {
        return None;
    };
    (version.axis() == PhysicalIntegrityVersionAxis::EnvelopeSchema
        && version.scope().artifact_family() == PhysicalIntegrityArtifactFamily::RootManifest)
        .then_some(version.observed())
}
