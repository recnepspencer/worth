use worth_runtime_bridge::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEvidenceFamily,
    BridgeCausalEvidenceReferenceIdentity, TruthCommitIdentity,
};

use super::super::super::super::*;
use super::super::materialization::*;

pub(super) fn admitted_artifact(
    commit_identity: TruthCommitIdentity,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> QueryCausalInspectionArtifact {
    admitted_artifact_for(
        commit_identity,
        CausalObservationOutcome::Changed,
        CausalInspectionReason::ChangedResult,
        resource_request,
    )
}

pub(super) fn admitted_artifact_for(
    commit_identity: TruthCommitIdentity,
    outcome: CausalObservationOutcome,
    reason: CausalInspectionReason,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> QueryCausalInspectionArtifact {
    let runtime = bridge_runtime();
    let routed = runtime.route(commit_identity, resource_request).unwrap();
    let reference_set = reference_set_for(routed.route_identity(), outcome, reason);
    let flow = admit_causal_inspection(request_for(
        reference_set,
        CausalInspectionRichness::ReferenceOnly,
    ));
    let CausalInspectionProofFlow::Admitted(admitted) = flow else {
        panic!("reference-only cross-runtime inspection should admit");
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
    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect("bridge envelope should assemble");

    materialize_admitted_causal_inspection(
        &admitted,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .expect("admitted materialization should consume bridge envelope")
}

pub(super) fn advisory_artifacts(
    commit_identity: TruthCommitIdentity,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (QueryCausalInspectionArtifact, QueryCausalInspectionArtifact) {
    let runtime = bridge_runtime();
    let routed = runtime.route(commit_identity, resource_request).unwrap();
    let reference_set = changed_reference_set(routed.route_identity());
    let flow = admit_causal_inspection(request_for(
        reference_set,
        CausalInspectionRichness::MaterializedDetail,
    ));
    let CausalInspectionProofFlow::Advisory(advisory) = flow else {
        panic!("materialized detail should narrow to advisory");
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
    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect("bridge envelope should assemble");
    let full = materialize_advisory_causal_inspection(
        &advisory,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .expect("full advisory materialization should consume bridge envelope");
    let redacted = materialize_advisory_causal_inspection(
        &advisory,
        &envelope,
        CausalInspectionRedactionPolicy::DigestOnly,
        CausalInspectionMaterializationPolicy::DigestReferenceOnly,
    )
    .expect("redacted advisory materialization should consume bridge envelope");
    (full, redacted)
}

pub(super) fn denied_artifact_and_missing_evidence(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (QueryCausalInspectionArtifact, String) {
    let runtime = bridge_runtime();
    let routed = runtime
        .route(
            super::super::causal_truth_commit_identity("commit-query-cert-denied"),
            resource_request,
        )
        .unwrap();
    let reference_set = changed_reference_set(routed.route_identity());
    let missing_resolution = resolve_causal_evidence_references(
        reference_set.anchor().clone(),
        &[CausalEvidenceFamily::SignalInvalidation],
    );
    let CausalEvidenceReferenceResolution::MissingRequiredEvidence { denial, .. } =
        missing_resolution
    else {
        panic!("signal evidence should be missing from bridge-route-only anchor");
    };
    let receipt = reference_set.anchor().observation_receipt();
    let target = causal_inspection_target(
        receipt.observation_target().clone(),
        receipt.result_shape_context().clone(),
    )
    .expect("target should match receipt");
    let request = request_causal_inspection(
        reference_set,
        target,
        CausalInspectionExplanationFamily::DurableCausalArchive,
        CausalInspectionRichness::ReferenceOnly,
        &[CausalEvidenceFamily::BridgeRoute],
    )
    .expect("causal inspection request should reach admission boundary");
    let CausalInspectionProofFlow::Denied(denied) = admit_causal_inspection(request) else {
        panic!("durable causal archive should deny for phase-6 certification");
    };
    let artifact = materialize_denied_causal_inspection(
        &denied,
        None,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    );
    (artifact, denial.failure_digest().to_string())
}

fn reference_set_for(
    route_identity: &worth_runtime_bridge::facade::BridgeRouteIdentity,
    outcome: CausalObservationOutcome,
    reason: CausalInspectionReason,
) -> CausalEvidenceReferenceSet {
    let anchor = anchor_causal_observation(
        QueryObservationReceipt::fixture(
            outcome,
            vec![
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::QueryInspection,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        format!("query-inspection:{}", outcome.as_str()),
                    ),
                ),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::BridgeRoute,
                    route_identity.bridge_admission_evidence(),
                ),
            ],
        ),
        reason,
    )
    .expect("fixture receipt should anchor");
    let CausalEvidenceReferenceResolution::Resolved { reference_set, .. } =
        resolve_causal_evidence_references(
            anchor,
            &[
                CausalEvidenceFamily::QueryInspection,
                CausalEvidenceFamily::BridgeRoute,
            ],
        )
    else {
        panic!("fixture references should resolve");
    };
    reference_set
}
