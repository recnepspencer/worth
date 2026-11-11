use crate::correspondence::{
    resolve_correspondence_evidence, CorrespondenceEvaluationRequest,
    StructuralCandidateDiscoveryPlan, StructuralCandidateOrderingContract,
};
use crate::correspondence_history::{
    compose_correspondence_historical_envelope, CorrespondenceHistoricalEnvelope,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::facade::foundation::{
    admit_historical_evaluation_path, resolve_historical_materialization_path,
    HistoricalCapabilityDescriptor, HistoricalEvaluationRequest,
    HistoricalMaterializationDescriptor, HistoricalPathReuseDescriptor,
    ResolvedHistoricalPathClass,
};

use super::paths::{reconstruction_resolved, replay_resolved, retained_resolved};
use super::preflight::{collection_preflight_bundle, detail_preflight_bundle};

pub(crate) fn correspondence_denied_envelope(
) -> Result<CorrespondenceHistoricalEnvelope, IdentityDerivationDenial> {
    let execution = crate::execution::execute_preflight_bundle(&detail_preflight_bundle())
        .expect("execution should succeed");
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
    let resolved = resolve_historical_materialization_path(
        admission,
        HistoricalMaterializationDescriptor::new_for_test(
            "basis:a",
            ResolvedHistoricalPathClass::ResolvedRetainedSnapshotPath,
        )?,
    )
    .expect("resolution should succeed");

    Ok(compose_correspondence_historical_envelope(
        execution,
        correspondence,
        resolved,
    ))
}

pub(crate) fn lineage_authoritative_envelope(
) -> Result<CorrespondenceHistoricalEnvelope, IdentityDerivationDenial> {
    let execution = crate::execution::execute_preflight_bundle(&detail_preflight_bundle())
        .expect("execution should succeed");
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::lineage_only(
            "subject:a",
            "record:a",
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            1,
        ))
        .expect("correspondence should resolve");
    Ok(compose_correspondence_historical_envelope(
        execution,
        correspondence,
        retained_resolved("basis:a")?,
    ))
}

pub(crate) fn structural_unique_replay_envelope(
) -> Result<CorrespondenceHistoricalEnvelope, IdentityDerivationDenial> {
    let execution = crate::execution::execute_preflight_bundle(&detail_preflight_bundle())
        .expect("execution should succeed");
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::structural_only(
            vec!["record:structural".into()],
            StructuralCandidateDiscoveryPlan::FingerprintBucketBounded,
            2,
            StructuralCandidateOrderingContract::StableFingerprintOrder,
        ))
        .expect("correspondence should resolve");
    Ok(compose_correspondence_historical_envelope(
        execution,
        correspondence,
        replay_resolved("basis:replay")?,
    ))
}

pub(crate) fn disagreement_envelope(
) -> Result<CorrespondenceHistoricalEnvelope, IdentityDerivationDenial> {
    let execution = crate::execution::execute_preflight_bundle(&detail_preflight_bundle())
        .expect("execution should succeed");
    let correspondence = resolve_correspondence_evidence(CorrespondenceEvaluationRequest::mixed(
        "subject:a",
        "record:a",
        vec!["record:z".into()],
        StructuralCandidateDiscoveryPlan::FingerprintBucketBounded,
        2,
        StructuralCandidateOrderingContract::StableFingerprintThenLineageHintOrder,
    ))
    .expect("correspondence should resolve");
    Ok(compose_correspondence_historical_envelope(
        execution,
        correspondence,
        retained_resolved("basis:a")?,
    ))
}

pub(crate) fn ambiguity_envelope(
) -> Result<CorrespondenceHistoricalEnvelope, IdentityDerivationDenial> {
    let execution = crate::execution::execute_preflight_bundle(&collection_preflight_bundle())
        .expect("execution should succeed");
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::structural_only(
            vec!["record:a".into(), "record:b".into()],
            StructuralCandidateDiscoveryPlan::FingerprintBucketBounded,
            4,
            StructuralCandidateOrderingContract::StableFingerprintOrder,
        ))
        .expect("correspondence should resolve");
    Ok(compose_correspondence_historical_envelope(
        execution,
        correspondence,
        replay_resolved("basis:replay")?,
    ))
}

pub(crate) fn retained_path_envelope(
) -> Result<CorrespondenceHistoricalEnvelope, IdentityDerivationDenial> {
    let execution = crate::execution::execute_preflight_bundle(&detail_preflight_bundle())
        .expect("execution should succeed");
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::lineage_only(
            "subject:a",
            "record:a",
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            1,
        ))
        .expect("correspondence should resolve");
    Ok(compose_correspondence_historical_envelope(
        execution,
        correspondence,
        retained_resolved("basis:a")?,
    ))
}

pub(crate) fn replay_path_envelope(
) -> Result<CorrespondenceHistoricalEnvelope, IdentityDerivationDenial> {
    let execution = crate::execution::execute_preflight_bundle(&detail_preflight_bundle())
        .expect("execution should succeed");
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::lineage_only(
            "subject:a",
            "record:a",
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            1,
        ))
        .expect("correspondence should resolve");
    Ok(compose_correspondence_historical_envelope(
        execution,
        correspondence,
        replay_resolved("basis:replay")?,
    ))
}

pub(crate) fn reconstruction_path_envelope(
) -> Result<CorrespondenceHistoricalEnvelope, IdentityDerivationDenial> {
    let execution = crate::execution::execute_preflight_bundle(&detail_preflight_bundle())
        .expect("execution should succeed");
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::lineage_only(
            "subject:a",
            "record:a",
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            1,
        ))
        .expect("correspondence should resolve");
    Ok(compose_correspondence_historical_envelope(
        execution,
        correspondence,
        reconstruction_resolved("basis:reconstruct")?,
    ))
}
