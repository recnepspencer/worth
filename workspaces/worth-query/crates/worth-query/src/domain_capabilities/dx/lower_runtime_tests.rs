use super::{test_support::success, WorthQueryDomainCapabilityOutcomeKind};
use crate::domain_capabilities::certification::install_domain_capability_certification;
use crate::domain_capabilities::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution,
    materialize_graph_composition_domain_invariant_denial,
    materialize_lower_runtime_support_traceability_artifact,
    materialize_query_causal_inspection_artifact, materialize_query_causal_inspection_review,
    prepare_admitted_domain_capability_contribution_for_materialization,
    WorthQueryExplanationContributionAuthoring, WorthQueryInvariantCapabilityContributionAuthoring,
    WorthQueryLowerRuntimeExplanationRequest, WorthQuerySupportContributionAuthoring,
};
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeRoutePlan, WorthQueryLowerRuntimeRouteSubjectIdentity,
    WorthQueryLowerRuntimeSeamKey,
};
use crate::runtime::{
    anchor_causal_observation, causal_inspection_target, resolve_causal_evidence_references,
    CausalEvidenceFamily, CausalEvidenceReferenceResolution, CausalInspectionMaterializationPolicy,
    CausalInspectionReason, CausalInspectionRedactionPolicy,
    WorthQueryGraphCompositionDomainInvariantDenial, WorthQueryReadExecutionEngine,
    WorthQueryReadReceipt, WorthQueryWriteCommand,
};

#[test]
fn common_lower_runtime_support_lane_matches_proof_lane_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let envelope = lower_runtime_envelope("boundary-support");
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let target = domain
        .lower_runtime_target(&envelope)
        .expect("installed contribution authority must remain current");

    let common = domain
        .for_lower_runtime_target(target.clone())
        .expect("certification target should belong to its installed domain")
        .supports_boundary_traceability("routing.signal_invalidation")
        .because("lower-runtime routing narrows the supported authority seam")
        .materialize()
        .expect("lower-runtime support lane should materialize");

    let proof_requested = WorthQuerySupportContributionAuthoring::narrowed_support(
        "worth.spatial.routing.signal_invalidation",
        "lower-runtime routing narrows the supported authority seam",
    )
    .bind_to_installed_target(target);
    let proof_admitted = success(admit_eligible_domain_capability_contribution(success(
        evaluate_requested_domain_capability_contribution(proof_requested),
    )));
    let proof_target = proof_admitted.payload().target().clone();
    let proof = materialize_lower_runtime_support_traceability_artifact(success(
        prepare_admitted_domain_capability_contribution_for_materialization(
            proof_admitted,
            proof_target,
        ),
    ));

    assert_eq!(common, success(proof));
}

#[path = "lower_runtime_tests/common_lower_runtime_explanation_review_matches_proof_lane_revie.rs"]
mod common_lower_runtime_explanation_review_matches_proof_lane_revie;

#[test]
fn common_lower_runtime_explanation_artifact_matches_proof_lane_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let envelope = lower_runtime_envelope("boundary-explanation-artifact");
    let (reference_set, inspection_target) = replay_gap_inputs();
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let target = domain
        .lower_runtime_target(&envelope)
        .expect("installed contribution authority must remain current");

    let common = domain
        .for_lower_runtime_target(target.clone())
        .expect("certification target should belong to its installed domain")
        .explains_store_backed_replay_gap(
            "replay.store_gap",
            WorthQueryLowerRuntimeExplanationRequest::explains_store_backed_replay_gap(
                reference_set.clone(),
                inspection_target.clone(),
                vec![CausalEvidenceFamily::QueryInspection],
                CausalInspectionRedactionPolicy::PreserveDetail,
                CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
            ),
        )
        .because("store-backed replay still lacks the retained lower-runtime evidence lane")
        .materialize_artifact()
        .expect("lower-runtime explanation artifact should materialize");

    let proof_requested =
        WorthQueryExplanationContributionAuthoring::store_backed_replay_gap_explanation(
            "worth.spatial.replay.store_gap",
            "store-backed replay still lacks the retained lower-runtime evidence lane",
            reference_set,
            inspection_target,
            vec![CausalEvidenceFamily::QueryInspection],
            CausalInspectionRedactionPolicy::PreserveDetail,
            CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
        )
        .bind_to_installed_target(target);
    let proof_admitted = success(admit_eligible_domain_capability_contribution(success(
        evaluate_requested_domain_capability_contribution(proof_requested),
    )));
    let proof_target = proof_admitted.payload().target().clone();
    let proof = materialize_query_causal_inspection_artifact(success(
        prepare_admitted_domain_capability_contribution_for_materialization(
            proof_admitted,
            proof_target,
        ),
    ));

    assert_eq!(common, success(proof));
}

#[test]
fn checked_lower_runtime_review_lane_preserves_denied_metadata() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let envelope = lower_runtime_envelope("boundary-explanation-denial");
    let (reference_set, inspection_target) = replay_gap_inputs();

    let checked = install_domain_capability_certification(request_execution)
        .contributions()
        .for_lower_runtime_boundary_envelope(&envelope)
        .expect("installed contribution authority must remain current")
        .explains_store_backed_replay_gap(
            "replay.store_gap",
            WorthQueryLowerRuntimeExplanationRequest::explains_store_backed_replay_gap(
                reference_set,
                inspection_target,
                vec![CausalEvidenceFamily::QueryInspection],
                CausalInspectionRedactionPolicy::PreserveDetail,
                CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
            ),
        )
        .because("")
        .try_review();

    assert_eq!(
        checked.kind(),
        WorthQueryDomainCapabilityOutcomeKind::Denied
    );
    assert_eq!(checked.category(), "explanation-inspection");
    assert_eq!(checked.semantic_posture(), "explains-ambiguity");
    assert_eq!(
        checked.target_kind(),
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::LowerRuntimeBoundaryEnvelope
    );
    assert!(checked.denial().is_some());
}

#[test]
fn lower_runtime_source_support_binding_matches_envelope_binding() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = write_authority_boundary_source("boundary-source-support");
    let installation = install_domain_capability_certification(request_execution);
    let domain = installation.contributions();
    let source_artifact = domain
        .for_lower_runtime_boundary_source(&source)
        .expect("installed contribution authority must remain current")
        .supports_boundary_traceability("routing.write_authority")
        .because("write authority receipt carries the lower-runtime boundary envelope")
        .materialize()
        .expect("source-bound support should materialize");
    let envelope_artifact = domain
        .for_lower_runtime_boundary_envelope(source.boundary_envelope())
        .expect("installed contribution authority must remain current")
        .supports_boundary_traceability("routing.write_authority")
        .because("write authority receipt carries the lower-runtime boundary envelope")
        .materialize()
        .expect("envelope-bound support should materialize");

    assert_eq!(source_artifact, envelope_artifact);
}

#[test]
fn lower_runtime_source_invariant_denial_matches_envelope_binding() {
    let source = write_authority_boundary_source("boundary-source-invariant");
    let source_denial = materialize_invariant_denial_from_source(&source);
    let envelope_denial = materialize_invariant_denial_from_envelope(source.boundary_envelope());

    assert_eq!(source_denial, envelope_denial);
}

fn lower_runtime_envelope(target_digest: &str) -> WorthQueryLowerRuntimeBoundaryEnvelope {
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        WorthQueryLowerRuntimeAuthorityOwner::Query,
        "signal-invalidation-routing",
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "domain-capabilities-dx-target",
        )
        .field_value(
            crate::evidence_identity::WorthQueryEvidenceTag::new("test_target"),
            target_digest,
        )
        .seal(),
    );
    let detail_identity = crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
        crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
    )
    .field_value(
        crate::evidence_identity::WorthQueryEvidenceTag::new("test_detail"),
        "detail",
    )
    .seal();
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request,
        &detail_identity,
    );
    let route = WorthQueryLowerRuntimeRoutePlan::new(
        eligibility,
        WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
            "domain-capabilities-dx-route",
            &detail_identity,
        ),
    );
    let retained_evidence =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "domain-capabilities-dx-test",
            &crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
                crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
            )
            .field_value(
                crate::evidence_identity::WorthQueryEvidenceTag::new("test_retained"),
                target_digest,
            )
            .seal(),
        );
    let boundary =
        WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan(&route, &retained_evidence);

    WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan(
        WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
        &route,
        &boundary,
        &retained_evidence,
    )
}

fn write_authority_boundary_source(
    target_digest: &str,
) -> crate::runtime::WriteAuthorityExecutionReceipt {
    use worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts;

    let command = WorthQueryWriteCommand::Delete {
        entity_identity: crate::memory_workspace::admit_authored_entity_label(target_digest),
    };
    let mutation_receipt =
        crate::memory_workspace::WorthQueryMutationReceipt::from_authoritative_parts(
            crate::memory_workspace::WorthQueryCommitIdentity::preview(
                crate::WorthQueryEvidenceIdentity::compose(
                    crate::WorthQueryEvidenceScope::WriteReceiptCommitIdentity,
                )
                .field_value(crate::WorthQueryEvidenceTag::new("target"), target_digest)
                .seal(),
            ),
            crate::memory_workspace::WorthQuerySnapshotIdentity::from_relational_snapshot(
                RelationalBridgeSnapshotIdentityParts::new(1, 1),
            ),
            Vec::new(),
        );
    crate::runtime::WriteAuthorityExecutionReceipt::from_command(&command, mutation_receipt)
}

fn materialize_invariant_denial_from_source<S>(
    source: &S,
) -> WorthQueryGraphCompositionDomainInvariantDenial
where
    S: crate::runtime::WorthQueryLowerRuntimeBoundaryEnvelopeSource + ?Sized,
{
    let requested = graph_invariant_denial_authoring().for_lower_runtime_boundary_source(source);
    materialize_invariant_denial(requested)
}

fn materialize_invariant_denial_from_envelope(
    envelope: &WorthQueryLowerRuntimeBoundaryEnvelope,
) -> WorthQueryGraphCompositionDomainInvariantDenial {
    let requested =
        graph_invariant_denial_authoring().for_lower_runtime_boundary_envelope(envelope);
    materialize_invariant_denial(requested)
}

fn graph_invariant_denial_authoring() -> WorthQueryInvariantCapabilityContributionAuthoring {
    WorthQueryInvariantCapabilityContributionAuthoring::graph_invariant_denial(
        "graph.non_manifold_edge_split",
        ["edges"],
        ["edge-1"],
        ["edge_split"],
        ["publication"],
        "program-digest",
        "breadth-digest",
        "counter-snapshot",
        "graph.non_manifold_edge_split",
        "edge split would violate graph composition invariant",
    )
}

fn materialize_invariant_denial(
    requested: crate::domain_capabilities::WorthQueryRequestedInvariantCapabilityContribution<
        crate::domain_capabilities::WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    >,
) -> WorthQueryGraphCompositionDomainInvariantDenial {
    let eligible = success(evaluate_requested_domain_capability_contribution(requested));
    let admitted = success(admit_eligible_domain_capability_contribution(eligible));
    let target = admitted.payload().target().clone();
    success(materialize_graph_composition_domain_invariant_denial(
        success(
            prepare_admitted_domain_capability_contribution_for_materialization(admitted, target),
        ),
    ))
}

fn replay_gap_inputs() -> (
    crate::runtime::CausalEvidenceReferenceSet,
    crate::runtime::CausalInspectionTarget,
) {
    let read_receipt = WorthQueryReadReceipt::test_only(
        "read-graph:domain-capability",
        "query:domain-capability",
        "basis:domain-capability",
        "result:domain-capability",
        WorthQueryReadExecutionEngine::QueryRuntimeHistorical,
    );
    let observation = crate::runtime::QueryObservationReceipt::from_read_receipt(
        &read_receipt,
        crate::basis_lifecycle::basis_lifecycle()
            .historical_snapshot("domain-capability-lower-runtime", true)
            .inspect()
            .unwrap(),
    );
    let anchor = anchor_causal_observation(
        observation.clone(),
        CausalInspectionReason::HistoricalReplayResult,
    )
    .expect("historical replay observation should anchor");
    let CausalEvidenceReferenceResolution::Resolved { reference_set, .. } =
        resolve_causal_evidence_references(anchor, &[CausalEvidenceFamily::QueryInspection])
    else {
        panic!("query-inspection-only replay evidence should resolve");
    };
    let target = causal_inspection_target(
        observation.observation_target().clone(),
        observation.result_shape_context().clone(),
    )
    .expect("observation-derived target should be valid");

    (reference_set, target)
}
