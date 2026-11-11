use worth_runtime_bridge::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEnvelopeDenialKind,
    BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner, BridgeCausalEvidenceReferenceIdentity,
    BridgeCausalInspectionAdmissionSummary, BridgeIdentityEvidence,
};

use super::super::super::super::*;
use super::support::*;

fn admitted_replay_flow_requesting_signal_cursor(
    route_identity: &worth_runtime_bridge::facade::BridgeRouteIdentity,
    signal_replay_cursor: &str,
    richness: CausalInspectionRichness,
) -> CausalInspectionProofFlow {
    let reference_set =
        replay_reference_set_with_signal_cursor(route_identity, signal_replay_cursor);
    admit_causal_inspection(request_for_families(
        reference_set,
        richness,
        &[
            CausalEvidenceFamily::BridgeRoute,
            CausalEvidenceFamily::SignalReplayCursor,
        ],
    ))
    .expect("fixture digest must fit")
}

fn bridge_route_only_envelope_for_admitted_replay(
    runtime: &worth_runtime_bridge::facade::RuntimeBridge,
    admitted: &AdmittedCausalInspection,
    routed: &worth_runtime_bridge::facade::BridgeRoute,
) -> worth_runtime_bridge::facade::BridgeCausalExplanationEnvelope {
    let summary = BridgeCausalInspectionAdmissionSummary::admitted(
        bridge_query_evidence(
            "causal-inspection-outcome",
            admitted.admitted_inspection_digest(),
        ),
        bridge_query_evidence(
            "causal-observation-anchor",
            admitted.subject().anchor_for_reporting(),
        ),
    )
    .expect("query admission summary should be valid");
    bridge_route_only_envelope(
        runtime,
        summary,
        admitted
            .subject()
            .query_observation_bridge_evidence_identity(),
        routed,
    )
}

fn bridge_route_only_envelope_for_advisory_replay(
    runtime: &worth_runtime_bridge::facade::RuntimeBridge,
    advisory: &AdvisoryCausalInspection,
    routed: &worth_runtime_bridge::facade::BridgeRoute,
) -> worth_runtime_bridge::facade::BridgeCausalExplanationEnvelope {
    let summary = BridgeCausalInspectionAdmissionSummary::advisory(
        bridge_query_evidence(
            "causal-inspection-outcome",
            advisory.advisory_inspection_digest(),
        ),
        bridge_query_evidence(
            "causal-observation-anchor",
            advisory.subject().anchor_for_reporting(),
        ),
    )
    .expect("query advisory summary should be valid");
    bridge_route_only_envelope(
        runtime,
        summary,
        advisory
            .subject()
            .query_observation_bridge_evidence_identity(),
        routed,
    )
}

fn signal_replay_cursor_envelope_for_admitted_replay(
    runtime: &worth_runtime_bridge::facade::RuntimeBridge,
    admitted: &AdmittedCausalInspection,
    routed: &worth_runtime_bridge::facade::BridgeRoute,
    signal_replay_cursor: &str,
) -> worth_runtime_bridge::facade::BridgeCausalExplanationEnvelope {
    let summary = BridgeCausalInspectionAdmissionSummary::admitted(
        bridge_query_evidence(
            "causal-inspection-outcome",
            admitted.admitted_inspection_digest(),
        ),
        bridge_query_evidence(
            "causal-observation-anchor",
            admitted.subject().anchor_for_reporting(),
        ),
    )
    .expect("query admission summary should be valid");
    let bridge_request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary,
        vec![
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    admitted
                        .subject()
                        .query_observation_bridge_evidence_identity(),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_reference(
                BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
                    BridgeCausalEvidenceFamily::BridgeRoute,
                    routed.route_identity().bridge_admission_evidence(),
                )
                .expect("route evidence reference identity should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalReplayCursor,
                    bridge_evidence(signal_replay_cursor),
                )
                .expect("signal replay cursor reference identity should be valid"),
            ),
        ],
    )
    .expect("bridge request should be valid");
    runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect("bridge envelope should assemble")
}

fn bridge_route_only_envelope(
    runtime: &worth_runtime_bridge::facade::RuntimeBridge,
    summary: BridgeCausalInspectionAdmissionSummary,
    query_observation_identity: BridgeIdentityEvidence,
    routed: &worth_runtime_bridge::facade::BridgeRoute,
) -> worth_runtime_bridge::facade::BridgeCausalExplanationEnvelope {
    let bridge_request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary,
        vec![
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    query_observation_identity,
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_reference(
                BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
                    BridgeCausalEvidenceFamily::BridgeRoute,
                    routed.route_identity().bridge_admission_evidence(),
                )
                .expect("route evidence reference identity should be valid"),
            ),
        ],
    )
    .expect("bridge request should be valid");
    runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect("bridge envelope should assemble")
}

fn bridge_evidence(value: impl AsRef<str>) -> BridgeIdentityEvidence {
    crate::runtime::tests::causal_inspection::bridge_external_evidence(value)
}

fn bridge_query_evidence(scope: &str, token: &str) -> BridgeIdentityEvidence {
    crate::runtime::tests::causal_inspection::bridge_query_evidence(scope, token)
}

fn summary_for_admitted(
    admitted: &AdmittedCausalInspection,
) -> BridgeCausalInspectionAdmissionSummary {
    BridgeCausalInspectionAdmissionSummary::admitted(
        bridge_query_evidence(
            "causal-inspection-outcome",
            admitted.admitted_inspection_digest(),
        ),
        bridge_query_evidence(
            "causal-observation-anchor",
            admitted.subject().anchor_for_reporting(),
        ),
    )
    .expect("query admission summary should be valid")
}

mod admitted_contracts;
mod advisory_contracts;
mod replay_cursor;
