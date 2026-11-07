use super::{
    build_correspondence_historical_parity_bundle, CorrespondenceHistoricalParityBundleError,
};
use crate::correspondence::{
    resolve_correspondence_evidence, CorrespondenceEvaluationRequest,
    StructuralCandidateDiscoveryPlan, StructuralCandidateOrderingContract,
};
use crate::correspondence_history::{
    compose_correspondence_historical_envelope, compose_historical_admission_denied_envelope,
    CorrespondenceHistoricalEnvelope,
};
use crate::execution::execute_preflight_bundle;
use crate::historical::{
    admit_historical_evaluation_path, resolve_historical_materialization_path,
    AdmittedHistoricalPathClass, HistoricalCapabilityDescriptor, HistoricalEvaluationRequest,
    HistoricalMaterializationDescriptor, HistoricalPathReuseDescriptor,
    RequestedHistoricalPathClass, ResolvedHistoricalPathClass,
};

#[test]
fn ambiguity_and_disagreement_bundle_digests_stay_distinct() {
    let ambiguity =
        build_correspondence_historical_parity_bundle(&ambiguous_envelope(), None, None)
            .expect("ambiguity bundle should build");
    let disagreement =
        build_correspondence_historical_parity_bundle(&disagreement_envelope(), None, None)
            .expect("disagreement bundle should build");

    assert_eq!(ambiguity.parity_variant().as_str(), "ambiguity");
    assert_eq!(disagreement.parity_variant().as_str(), "disagreement");
    assert_ne!(
        ambiguity.correspondence_outcome_digest().as_str(),
        disagreement.correspondence_outcome_digest().as_str()
    );
    assert_eq!(
        ambiguity.performance_prediction_drift_outcome().as_str(),
        "within_budget"
    );
}

#[test]
fn retained_and_replay_paths_digest_differ() {
    let retained = build_correspondence_historical_parity_bundle(
        &success_envelope_for(
            RequestedHistoricalPathClass::RequestedRetainedSnapshotPath,
            AdmittedHistoricalPathClass::AdmittedRetainedSnapshotPath,
            ResolvedHistoricalPathClass::ResolvedRetainedSnapshotPath,
            HistoricalPathReuseDescriptor::retained_reuse(),
        ),
        None,
        None,
    )
    .expect("retained bundle should build");
    let replay = build_correspondence_historical_parity_bundle(
        &success_envelope_for(
            RequestedHistoricalPathClass::RequestedDeltaReplayPath,
            AdmittedHistoricalPathClass::AdmittedDeltaReplayPath,
            ResolvedHistoricalPathClass::ResolvedDeltaReplayPath,
            HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
        ),
        None,
        None,
    )
    .expect("replay bundle should build");

    assert_ne!(
        retained
            .requested_path_digest()
            .expect("retained path")
            .as_str(),
        replay
            .requested_path_digest()
            .expect("replay path")
            .as_str()
    );
    assert_ne!(
        retained
            .historical_cost_posture_digest()
            .expect("retained posture")
            .as_str(),
        replay
            .historical_cost_posture_digest()
            .expect("replay posture")
            .as_str()
    );
}

#[test]
fn historical_denial_bundle_reports_failure_digest() {
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
    .expect_err("admission should fail");
    let envelope =
        compose_historical_admission_denied_envelope(correspondence, denied_request, error.clone());
    let preflight = detail_preflight_bundle();
    let bundle = build_correspondence_historical_parity_bundle(
        &envelope,
        Some(preflight.plan().query().validated_query_digest().clone()),
        Some(preflight.basis().proof().digest().clone()),
    )
    .expect("denied bundle should build");

    assert_eq!(bundle.parity_variant().as_str(), "historical_path_denied");
    assert!(bundle.result_digest().is_none());
    assert!(bundle.admitted_path_digest().is_none());
    assert_eq!(
        bundle
            .requested_path_digest()
            .expect("requested path digest")
            .as_str(),
        crate::identity::HistoricalPathClassDigest::from_parts(&[
            "requested_path:requested_delta_replay_path".to_string(),
        ])
        .as_str()
    );
    assert_eq!(
        bundle
            .failure_digest()
            .expect("denial bundle should emit failure digest")
            .as_str(),
        super::digests::digest_historical_failure(&error).as_str()
    );
    assert_eq!(
        bundle
            .historical_compatibility_outcome()
            .expect("compatibility outcome")
            .as_str(),
        "denied"
    );
}

#[test]
fn denied_bundle_requires_query_and_basis_digest_overrides() {
    let envelope = correspondence_denied_envelope();
    let error = build_correspondence_historical_parity_bundle(&envelope, None, None).unwrap_err();
    let preflight = detail_preflight_bundle();
    let missing_basis = build_correspondence_historical_parity_bundle(
        &envelope,
        Some(preflight.plan().query().validated_query_digest().clone()),
        None,
    )
    .unwrap_err();

    assert_eq!(
        error,
        CorrespondenceHistoricalParityBundleError::MissingDeniedQueryDigest
    );
    assert_eq!(
        missing_basis,
        CorrespondenceHistoricalParityBundleError::MissingDeniedBasisDigest
    );
}

mod correspondence_denied_bundle_stays_pathless;

fn success_envelope_for(
    requested: RequestedHistoricalPathClass,
    admitted: AdmittedHistoricalPathClass,
    resolved: ResolvedHistoricalPathClass,
    reuse: HistoricalPathReuseDescriptor,
) -> CorrespondenceHistoricalEnvelope {
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
    let request = match requested {
        RequestedHistoricalPathClass::RequestedRetainedSnapshotPath => {
            HistoricalEvaluationRequest::retained_snapshot_for_test("basis:a", 1, 1, reuse)
        }
        RequestedHistoricalPathClass::RequestedDeltaReplayPath => {
            HistoricalEvaluationRequest::delta_replay_for_test("basis:a", 4, 8, reuse)
        }
        RequestedHistoricalPathClass::RequestedFullReconstructionPath => {
            HistoricalEvaluationRequest::full_reconstruction_for_test("basis:a", 4, 8, reuse)
        }
    }
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        "basis:a",
        Some(admitted),
        true,
        false,
        true,
        true,
        request.reuse_descriptor().clone(),
    )
    .expect("fixture digest must fit");
    let admission =
        admit_historical_evaluation_path(request, capability).expect("admission should succeed");
    let resolved = resolve_historical_materialization_path(
        admission,
        HistoricalMaterializationDescriptor::new_for_test("basis:a", resolved)
            .expect("fixture digest must fit"),
    )
    .expect("resolution should succeed");

    compose_correspondence_historical_envelope(execution, correspondence, resolved)
}

fn ambiguous_envelope() -> CorrespondenceHistoricalEnvelope {
    let execution =
        execute_preflight_bundle(&collection_preflight_bundle()).expect("execution should succeed");
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
        Some(AdmittedHistoricalPathClass::AdmittedDeltaReplayPath),
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

    compose_correspondence_historical_envelope(execution, correspondence, resolved)
}

fn disagreement_envelope() -> CorrespondenceHistoricalEnvelope {
    let execution =
        execute_preflight_bundle(&detail_preflight_bundle()).expect("execution should succeed");
    let correspondence = resolve_correspondence_evidence(CorrespondenceEvaluationRequest::mixed(
        "subject:a",
        "record:a",
        vec!["record:z".into()],
        StructuralCandidateDiscoveryPlan::FingerprintBucketBounded,
        2,
        StructuralCandidateOrderingContract::StableFingerprintThenLineageHintOrder,
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
        Some(AdmittedHistoricalPathClass::AdmittedRetainedSnapshotPath),
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

    compose_correspondence_historical_envelope(execution, correspondence, resolved)
}

fn correspondence_denied_envelope() -> CorrespondenceHistoricalEnvelope {
    let execution =
        execute_preflight_bundle(&detail_preflight_bundle()).expect("execution should succeed");
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::structural_only(
            vec!["record:a".into(), "record:b".into(), "record:c".into()],
            StructuralCandidateDiscoveryPlan::RequiresBroadScanDenied,
            2,
            StructuralCandidateOrderingContract::StableFingerprintOrder,
        ))
        .expect("correspondence should resolve into denial");
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        "basis:a",
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        "basis:a",
        Some(AdmittedHistoricalPathClass::AdmittedRetainedSnapshotPath),
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

    compose_correspondence_historical_envelope(execution, correspondence, resolved)
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
