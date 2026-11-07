use super::{
    compose_correspondence_historical_envelope, compose_historical_admission_denied_envelope,
    CorrespondenceHistoricalEnvelope,
};
use crate::correspondence::{
    resolve_correspondence_evidence, CorrespondenceEvaluationRequest,
    StructuralCandidateDiscoveryPlan, StructuralCandidateOrderingContract,
};
use crate::execution::execute_preflight_bundle;
use crate::historical::{
    admit_historical_evaluation_path, resolve_historical_materialization_path,
    HistoricalCapabilityDescriptor, HistoricalEvaluationRequest,
    HistoricalMaterializationDescriptor, HistoricalPathReuseDescriptor,
    ResolvedHistoricalPathClass,
};

#[test]
fn success_envelope_preserves_rows_and_metadata_together() {
    let execution =
        execute_preflight_bundle(&detail_preflight_bundle()).expect("execution should succeed");
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::lineage_only(
            "subject:a",
            "record:a",
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            1,
        ))
        .expect("correspondence should resolve");
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        "basis:a",
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        "basis:a",
        Some(crate::historical::AdmittedHistoricalPathClass::AdmittedRetainedSnapshotPath),
        false,
        false,
        true,
        false,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let admission =
        admit_historical_evaluation_path(request, capability).expect("admission should succeed");
    let resolved = resolve_historical_materialization_path(
        admission,
        HistoricalMaterializationDescriptor::new_for_test(
            "basis:a",
            ResolvedHistoricalPathClass::ResolvedRetainedSnapshotPath,
        )
        .expect("fixture digest must fit"),
    )
    .expect("resolution should succeed");

    let envelope =
        compose_correspondence_historical_envelope(execution.clone(), correspondence, resolved);
    let view = envelope
        .result_view()
        .expect("success envelope should expose metadata-preserving view");

    assert_eq!(view.rows(), execution.rows());
    assert_eq!(view.correspondence_family_name(), "lineage_continuity");
    assert_eq!(
        view.materialization_metadata()
            .resolved_path_class()
            .as_str(),
        "resolved_retained_snapshot_path"
    );
}

#[test]
fn ambiguity_envelope_still_requires_metadata_preserving_view() {
    let execution = execute_preflight_bundle(&collection_preflight_bundle())
        .expect("collection execution should succeed");
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::structural_only(
            vec!["record:a".into(), "record:b".into()],
            StructuralCandidateDiscoveryPlan::FingerprintBucketBounded,
            4,
            StructuralCandidateOrderingContract::StableFingerprintOrder,
        ))
        .expect("correspondence should resolve");
    let request = HistoricalEvaluationRequest::delta_replay_for_test(
        "basis:replay",
        4,
        8,
        HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
    )
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        "basis:replay",
        Some(crate::historical::AdmittedHistoricalPathClass::AdmittedDeltaReplayPath),
        true,
        false,
        false,
        true,
        HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
    )
    .expect("fixture digest must fit");
    let admission =
        admit_historical_evaluation_path(request, capability).expect("admission should succeed");
    let resolved = resolve_historical_materialization_path(
        admission,
        HistoricalMaterializationDescriptor::new_for_test(
            "basis:replay",
            ResolvedHistoricalPathClass::ResolvedDeltaReplayPath,
        )
        .expect("fixture digest must fit"),
    )
    .expect("resolution should succeed");

    let envelope = compose_correspondence_historical_envelope(execution, correspondence, resolved);

    match envelope {
        CorrespondenceHistoricalEnvelope::Ambiguity(ref ambiguity) => {
            assert_eq!(
                ambiguity.result_view().correspondence_family_name(),
                "advisory_structural_ambiguous"
            );
        }
        _ => panic!("expected ambiguity envelope"),
    }
}

#[test]
fn path_denied_envelope_carries_typed_denial_without_payload() {
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::lineage_only(
            "subject:a",
            "record:a",
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            1,
        ))
        .expect("correspondence should resolve");
    let denied_request = HistoricalEvaluationRequest::delta_replay_for_test(
        "basis:replay",
        2,
        2,
        HistoricalPathReuseDescriptor::no_reuse(),
    )
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        "basis:replay",
        None,
        false,
        false,
        true,
        false,
        HistoricalPathReuseDescriptor::no_reuse(),
    )
    .expect("fixture digest must fit");
    let error = admit_historical_evaluation_path(
        denied_request.clone().expect("fixture digest must fit"),
        capability,
    )
    .expect_err("admission should fail for denied replay");
    let envelope =
        compose_historical_admission_denied_envelope(correspondence, denied_request, error);
    assert!(envelope.result_view().is_none());
    match envelope {
        CorrespondenceHistoricalEnvelope::HistoricalPathAdmissionDenied(ref denied) => {
            assert_eq!(
                denied.error().failure_class(),
                crate::historical::HistoricalEvaluationFailureClass::ReplayNotPermitted
            );
            assert_eq!(denied.compatibility_outcome().as_str(), "denied");
        }
        _ => panic!("expected historical path denied envelope"),
    }
}

fn detail_preflight_bundle() -> crate::basis::ExecutionPreflightBundle {
    let validated = crate::harness::fixtures::validated_bundles::runtime_detail_bundle();
    let request = crate::harness::fixtures::planning_requests::direct_runtime_request(&validated);
    let basis = crate::harness::fixtures::resolved_bases::runtime_basis(
        &validated,
        &crate::harness::fixtures::resolved_bases::primary_snapshot_identity(),
    );
    let plan = crate::facade::policy::plan_validated_bundle(&validated, request)
        .expect("detail validated bundle should plan");
    crate::facade::foundation::preflight_execution_basis(plan, basis)
        .expect("detail plan should preflight")
}

fn collection_preflight_bundle() -> crate::basis::ExecutionPreflightBundle {
    let validated = crate::harness::fixtures::validated_bundles::ordered_collection_bundle();
    let request = crate::harness::fixtures::planning_requests::direct_runtime_request(&validated);
    let basis = crate::harness::fixtures::resolved_bases::runtime_basis(
        &validated,
        &crate::harness::fixtures::resolved_bases::primary_snapshot_identity(),
    );
    let plan = crate::facade::policy::plan_validated_bundle(&validated, request)
        .expect("collection validated bundle should plan");
    crate::facade::foundation::preflight_execution_basis(plan, basis)
        .expect("collection plan should preflight")
}
