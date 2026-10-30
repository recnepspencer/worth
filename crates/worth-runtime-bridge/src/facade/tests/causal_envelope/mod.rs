use super::*;
use crate::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEnvelopeDenialKind,
    BridgeCausalEvidenceBinding, BridgeCausalEvidenceBindingClass, BridgeCausalEvidenceFamily,
    BridgeCausalEvidenceOwner, BridgeCausalEvidenceReference,
    BridgeCausalEvidenceReferenceIdentity, BridgeTruthViewEvaluationRequest,
};

mod admission_summary;
mod authority_boundary;
mod external_authority;
mod lookup_cost;
mod mapping;
mod mapping_scale;
mod mapping_support;
mod query_observation_reporting;
mod receipt;
mod request_anchor_validation;
mod retained_mapping;
mod retained_mapping_bulk;
mod retained_mapping_digest_support;
mod retained_mapping_edges;
mod retained_mapping_stream_history;
mod retained_mapping_support;
mod retained_record_denials;

use retained_mapping_digest_support::{
    expected_retained_causal_digest, ExpectedRetainedCausalDigestArtifact,
};

fn bridge_reference(
    identity: BridgeCausalEvidenceReferenceIdentity,
) -> BridgeCausalEvidenceReference {
    let family = identity.family();
    BridgeCausalEvidenceReference::new(BridgeCausalEvidenceOwner::RuntimeBridge, family, identity)
        .expect("bridge reference should be valid")
}

fn external_reference(
    owner: BridgeCausalEvidenceOwner,
    identity: BridgeCausalEvidenceReferenceIdentity,
) -> BridgeCausalEvidenceReference {
    let family = identity.family();
    BridgeCausalEvidenceReference::new(owner, family, identity)
        .expect("external reference should be valid")
}

fn query_observation_reference(
    identity: BridgeCausalEvidenceReferenceIdentity,
) -> BridgeCausalEvidenceReference {
    external_reference(BridgeCausalEvidenceOwner::Query, identity)
}

fn missing_bridge_reference(
    family: BridgeCausalEvidenceFamily,
    identity: &str,
) -> BridgeCausalEvidenceReference {
    bridge_reference(
        BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
            family,
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(identity),
        )
        .expect("bridge reference identity should be valid"),
    )
}

fn bridge_route_reference(
    route_summary: &crate::routing::BridgeRouteResultSummary,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeRoute,
        route_summary.route_identity().as_str(),
    )
}

fn bridge_historical_evaluation_reference(
    record: &crate::diagnostics::BridgeCanonicalHistoricalEvaluationRecord,
) -> BridgeCausalEvidenceReference {
    missing_bridge_reference(
        BridgeCausalEvidenceFamily::BridgeHistoricalEvaluation,
        record.record_identity().as_str(),
    )
}

fn binding_for<'a>(
    bindings: &'a [BridgeCausalEvidenceBinding],
    owner: BridgeCausalEvidenceOwner,
    family: BridgeCausalEvidenceFamily,
    reference_identity: &str,
) -> &'a BridgeCausalEvidenceBinding {
    bindings
        .iter()
        .find(|binding| {
            binding.owner() == owner
                && binding.family() == family
                && binding.reference_evidence_identity().as_str() == reference_identity
        })
        .expect("expected causal evidence binding should be present")
}

fn expected_retained_route_digest(
    route_identity: &str,
    invalidation_identity: &str,
    source_commit: &str,
) -> String {
    expected_retained_causal_digest(
        ExpectedRetainedCausalDigestArtifact::RouteRecord,
        &[route_identity, invalidation_identity, source_commit],
    )
}

fn expected_retained_historical_digest(
    record_identity: &str,
    decision_log_identity: &str,
    snapshot_identity: &str,
) -> String {
    expected_retained_causal_digest(
        ExpectedRetainedCausalDigestArtifact::HistoricalEvaluationRecord,
        &[record_identity, decision_log_identity, snapshot_identity],
    )
}
