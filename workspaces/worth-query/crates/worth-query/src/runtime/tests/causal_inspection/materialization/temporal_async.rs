use worth_runtime_bridge::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner,
    BridgeCausalEvidenceReferenceIdentity, BridgeIdentityEvidence,
};

use super::super::super::super::*;
use super::support::*;

fn reference_set_for(
    outcome: CausalObservationOutcome,
    reason: CausalInspectionReason,
    evidence_identities: Vec<CausalObservationEvidenceIdentity>,
    requested_families: &[CausalEvidenceFamily],
) -> CausalEvidenceReferenceSet {
    let anchor = anchor_causal_observation(
        QueryObservationReceipt::fixture(outcome, evidence_identities),
        reason,
    )
    .expect("temporal/async fixture should anchor");
    let CausalEvidenceReferenceResolution::Resolved { reference_set, .. } =
        resolve_causal_evidence_references(anchor, requested_families)
    else {
        panic!("temporal/async fixture references should resolve");
    };
    reference_set
}

#[test]
fn admitted_temporal_wake_materialization_projects_query_owned_temporal_explanation() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-query-temporal-wake"),
            resource_request,
        )
        .expect("temporal wake route should resolve");
    let reference_set = reference_set_for(
        CausalObservationOutcome::Changed,
        CausalInspectionReason::ChangedResult,
        vec![
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-inspection:temporal-wake",
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeRoute,
                routed.route_identity().bridge_admission_evidence(),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::SignalInvalidation,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "signal-invalidation:temporal-wake",
                ),
            ),
        ],
        &[
            CausalEvidenceFamily::BridgeRoute,
            CausalEvidenceFamily::SignalInvalidation,
        ],
    );
    let request = request_for_families(
        reference_set,
        CausalInspectionRichness::ReferenceOnly,
        &[
            CausalEvidenceFamily::BridgeRoute,
            CausalEvidenceFamily::SignalInvalidation,
        ],
    );
    let CausalInspectionProofFlow::Admitted(admitted) = admit_causal_inspection(request) else {
        panic!("reference-only temporal wake should admit");
    };
    let summary = crate::runtime::tests::causal_inspection::bridge_admitted_summary(&admitted);
    let bridge_request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary,
        vec![
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    admitted
                        .subject()
                        .query_observation_bridge_evidence_identity(),
                )
                .expect("query observation reference should be valid"),
            ),
            bridge_reference(
                BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
                    BridgeCausalEvidenceFamily::BridgeRoute,
                    routed.route_identity().bridge_admission_evidence(),
                )
                .expect("bridge route reference should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalInvalidation,
                    crate::runtime::tests::causal_inspection::bridge_external_evidence(
                        "signal-invalidation:temporal-wake",
                    ),
                )
                .expect("signal invalidation reference should be valid"),
            ),
        ],
    )
    .expect("temporal wake bridge request should be valid");
    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect("temporal wake envelope should assemble");

    let QueryCausalInspectionArtifact::Admitted(artifact) = materialize_admitted_causal_inspection(
        &admitted,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .expect("temporal wake materialization should succeed") else {
        panic!("expected admitted temporal wake artifact");
    };

    assert_eq!(
        artifact.temporal_async_explanation().kind(),
        QueryCausalTemporalAsyncExplanationKind::TemporalWake
    );
    assert!(artifact.temporal_async_explanation().offline_explainable());
}

#[test]
fn advisory_async_completion_materialization_projects_query_owned_async_explanation() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-query-async-completion"),
            resource_request,
        )
        .expect("async completion route should resolve");
    let reference_set = reference_set_for(
        CausalObservationOutcome::Changed,
        CausalInspectionReason::ChangedResult,
        vec![
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-inspection:async-completion",
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeRoute,
                routed.route_identity().bridge_admission_evidence(),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::SignalEvaluation,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "signal-evaluation:async-completion",
                ),
            ),
        ],
        &[
            CausalEvidenceFamily::BridgeRoute,
            CausalEvidenceFamily::SignalEvaluation,
        ],
    );
    let request = request_for_families(
        reference_set,
        CausalInspectionRichness::MaterializedDetail,
        &[
            CausalEvidenceFamily::BridgeRoute,
            CausalEvidenceFamily::SignalEvaluation,
        ],
    );
    let CausalInspectionProofFlow::Advisory(advisory) = admit_causal_inspection(request) else {
        panic!("materialized async completion should narrow to advisory");
    };
    let summary = crate::runtime::tests::causal_inspection::bridge_advisory_summary(&advisory);
    let bridge_request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary,
        vec![
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    advisory
                        .subject()
                        .query_observation_bridge_evidence_identity(),
                )
                .expect("query observation reference should be valid"),
            ),
            bridge_reference(
                BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
                    BridgeCausalEvidenceFamily::BridgeRoute,
                    routed.route_identity().bridge_admission_evidence(),
                )
                .expect("bridge route reference should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalEvaluation,
                    bridge_evidence("signal-evaluation:async-completion"),
                )
                .expect("signal evaluation reference should be valid"),
            ),
        ],
    )
    .expect("async completion bridge request should be valid");
    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect("async completion envelope should assemble");

    let QueryCausalInspectionArtifact::Advisory(artifact) = materialize_advisory_causal_inspection(
        &advisory,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .expect("async completion advisory materialization should succeed") else {
        panic!("expected advisory async completion artifact");
    };

    assert_eq!(
        artifact.temporal_async_explanation().kind(),
        QueryCausalTemporalAsyncExplanationKind::AsyncCompletion
    );
    assert!(artifact.temporal_async_explanation().offline_explainable());
}

#[test]
fn admitted_mixed_cause_suppression_materialization_retains_suppression_identity() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-query-mixed-suppressed"),
            resource_request,
        )
        .expect("mixed suppression route should resolve");
    let reference_set = reference_set_for(
        CausalObservationOutcome::Suppressed,
        CausalInspectionReason::SuppressedResult,
        vec![
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-inspection:mixed-suppressed",
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeRoute,
                routed.route_identity().bridge_admission_evidence(),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::SignalInvalidation,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "signal-invalidation:mixed-suppressed",
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::SignalEvaluation,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "signal-evaluation:mixed-suppressed",
                ),
            ),
        ],
        &[
            CausalEvidenceFamily::BridgeRoute,
            CausalEvidenceFamily::SignalInvalidation,
            CausalEvidenceFamily::SignalEvaluation,
        ],
    );
    let request = request_for_families(
        reference_set,
        CausalInspectionRichness::ReferenceOnly,
        &[
            CausalEvidenceFamily::BridgeRoute,
            CausalEvidenceFamily::SignalInvalidation,
            CausalEvidenceFamily::SignalEvaluation,
        ],
    );
    let CausalInspectionProofFlow::Admitted(admitted) = admit_causal_inspection(request) else {
        panic!("reference-only mixed suppression should admit");
    };
    let summary = crate::runtime::tests::causal_inspection::bridge_admitted_summary(&admitted);
    let bridge_request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary,
        vec![
            query_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    admitted
                        .subject()
                        .query_observation_bridge_evidence_identity(),
                )
                .expect("query observation reference should be valid"),
            ),
            bridge_reference(
                BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
                    BridgeCausalEvidenceFamily::BridgeRoute,
                    routed.route_identity().bridge_admission_evidence(),
                )
                .expect("bridge route reference should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalInvalidation,
                    bridge_evidence("signal-invalidation:mixed-suppressed"),
                )
                .expect("signal invalidation reference should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalEvaluation,
                    bridge_evidence("signal-evaluation:mixed-suppressed"),
                )
                .expect("signal evaluation reference should be valid"),
            ),
        ],
    )
    .expect("mixed suppression bridge request should be valid");
    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect("mixed suppression envelope should assemble");

    let artifact = materialize_admitted_causal_inspection(
        &admitted,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .expect("mixed suppression materialization should succeed");

    assert_eq!(
        artifact.temporal_async_explanation().kind(),
        QueryCausalTemporalAsyncExplanationKind::MixedCauseSuppression
    );
}

fn bridge_evidence(value: impl AsRef<str>) -> BridgeIdentityEvidence {
    crate::runtime::tests::causal_inspection::bridge_external_evidence(value)
}

mod request_parity;
