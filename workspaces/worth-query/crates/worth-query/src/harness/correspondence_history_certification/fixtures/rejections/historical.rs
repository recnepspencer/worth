use crate::correspondence::{
    resolve_correspondence_evidence, CorrespondenceEvaluationRequest,
    StructuralCandidateDiscoveryPlan,
};
use crate::correspondence_history::{
    compose_historical_admission_denied_envelope, compose_historical_path_denied_envelope,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::facade::foundation::{
    admit_historical_evaluation_path, build_correspondence_historical_parity_bundle,
    resolve_historical_materialization_path, HistoricalCapabilityDescriptor,
    HistoricalEvaluationError, HistoricalEvaluationRequest, HistoricalMaterializationDescriptor,
    HistoricalPathReuseDescriptor, RequestedHistoricalPathClass, ResolvedHistoricalPathClass,
};

use super::super::scenarios::detail_preflight_bundle;
use crate::harness::correspondence_history_certification::model::{
    CorrespondenceHistoryCertificationRejection, CorrespondenceHistoryFailureClass,
};

pub(crate) fn unsupported_historical_materialization_rejection(
) -> Result<CorrespondenceHistoryCertificationRejection, IdentityDerivationDenial> {
    let preflight = detail_preflight_bundle();
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
    )?;
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        "basis:a",
        Some(crate::historical::AdmittedHistoricalPathClass::AdmittedRetainedSnapshotPath),
        false,
        false,
        true,
        false,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )?;
    let admission =
        admit_historical_evaluation_path(request, capability).expect("admission should succeed");
    let error = HistoricalEvaluationError::UnsupportedBridgeMaterializationPath {
        requested_path_class: RequestedHistoricalPathClass::RequestedRetainedSnapshotPath,
        path_name: "unsupported_test_path",
    };
    let envelope =
        compose_historical_path_denied_envelope(correspondence, admission, error.clone());
    let parity_bundle = build_correspondence_historical_parity_bundle(
        &envelope,
        Some(preflight.plan().query().validated_query_digest().clone()),
        Some(preflight.basis().proof().digest().clone()),
    )
    .expect("unsupported historical path bundle should build");

    Ok(CorrespondenceHistoryCertificationRejection {
        failure_class: CorrespondenceHistoryFailureClass::HistoricalPathDenied,
        failure_digest: parity_bundle
            .failure_digest()
            .expect("historical denial should emit failure digest")
            .as_str()
            .to_string(),
        counter_snapshot_digest: Some(parity_bundle.counter_snapshot_digest().as_str().to_string()),
    })
}

pub(crate) fn hidden_materialization_substitution_rejection(
) -> Result<CorrespondenceHistoryCertificationRejection, IdentityDerivationDenial> {
    let preflight = detail_preflight_bundle();
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::lineage_only(
            "subject:a",
            "record:a",
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            1,
        ))
        .expect("correspondence should resolve");
    let request = HistoricalEvaluationRequest::delta_replay_for_test(
        "basis:replay",
        4,
        8,
        HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
    )?;
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        "basis:replay",
        Some(crate::historical::AdmittedHistoricalPathClass::AdmittedDeltaReplayPath),
        true,
        false,
        false,
        true,
        HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
    )?;
    let admission =
        admit_historical_evaluation_path(request, capability).expect("admission should succeed");
    let error = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::new_for_test(
            "basis:replay",
            ResolvedHistoricalPathClass::ResolvedRetainedSnapshotPath,
        )?,
    )
    .expect_err("path mutation should fail");
    let envelope = compose_historical_path_denied_envelope(correspondence, admission, error);
    let parity_bundle = build_correspondence_historical_parity_bundle(
        &envelope,
        Some(preflight.plan().query().validated_query_digest().clone()),
        Some(preflight.basis().proof().digest().clone()),
    )
    .expect("substitution denied bundle should build");

    Ok(CorrespondenceHistoryCertificationRejection {
        failure_class: CorrespondenceHistoryFailureClass::HistoricalPathDenied,
        failure_digest: parity_bundle
            .failure_digest()
            .expect("substitution denied should emit failure digest")
            .as_str()
            .to_string(),
        counter_snapshot_digest: Some(parity_bundle.counter_snapshot_digest().as_str().to_string()),
    })
}

pub(crate) fn executor_path_mutation_rejection(
) -> Result<CorrespondenceHistoryCertificationRejection, IdentityDerivationDenial> {
    let preflight = detail_preflight_bundle();
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::lineage_only(
            "subject:a",
            "record:a",
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            1,
        ))
        .expect("correspondence should resolve");
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        "basis:executor",
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )?;
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        "basis:executor",
        Some(crate::historical::AdmittedHistoricalPathClass::AdmittedRetainedSnapshotPath),
        false,
        false,
        true,
        false,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )?;
    let admission =
        admit_historical_evaluation_path(request, capability).expect("admission should succeed");
    let error = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::new_for_test(
            "basis:executor",
            ResolvedHistoricalPathClass::ResolvedFullReconstructionPath,
        )?,
    )
    .expect_err("executor mutation should fail");
    let envelope = compose_historical_path_denied_envelope(correspondence, admission, error);
    let parity_bundle = build_correspondence_historical_parity_bundle(
        &envelope,
        Some(preflight.plan().query().validated_query_digest().clone()),
        Some(preflight.basis().proof().digest().clone()),
    )
    .expect("executor mutation denial bundle should build");

    Ok(CorrespondenceHistoryCertificationRejection {
        failure_class: CorrespondenceHistoryFailureClass::HistoricalPathDenied,
        failure_digest: parity_bundle
            .failure_digest()
            .expect("executor mutation denial should emit failure digest")
            .as_str()
            .to_string(),
        counter_snapshot_digest: Some(parity_bundle.counter_snapshot_digest().as_str().to_string()),
    })
}

pub(crate) fn host_cache_history_authority_rejection(
) -> Result<CorrespondenceHistoryCertificationRejection, IdentityDerivationDenial> {
    let preflight = detail_preflight_bundle();
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::lineage_only(
            "subject:a",
            "record:a",
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            1,
        ))
        .expect("correspondence should resolve");
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        "basis:host-cache",
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )?;
    let error = HistoricalEvaluationError::UnsupportedHistoricalPathRequest {
        requested_path_class: RequestedHistoricalPathClass::RequestedRetainedSnapshotPath,
        reason: "historical evaluation authority may not be satisfied from host cache state",
    };
    let envelope = compose_historical_admission_denied_envelope(correspondence, request, error);
    let parity_bundle = build_correspondence_historical_parity_bundle(
        &envelope,
        Some(preflight.plan().query().validated_query_digest().clone()),
        Some(preflight.basis().proof().digest().clone()),
    )
    .expect("host cache authority denial bundle should build");

    Ok(CorrespondenceHistoryCertificationRejection {
        failure_class: CorrespondenceHistoryFailureClass::HistoricalPathDenied,
        failure_digest: parity_bundle
            .failure_digest()
            .expect("host cache denial should emit failure digest")
            .as_str()
            .to_string(),
        counter_snapshot_digest: Some(parity_bundle.counter_snapshot_digest().as_str().to_string()),
    })
}
