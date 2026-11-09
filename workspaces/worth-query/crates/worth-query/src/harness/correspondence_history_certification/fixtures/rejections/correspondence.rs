use crate::correspondence::{
    resolve_correspondence_evidence, CorrespondenceEvaluationRequest,
    StructuralCandidateDiscoveryPlan,
};
use crate::correspondence_history::compose_correspondence_historical_envelope;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::facade::foundation::build_correspondence_historical_parity_bundle;

use super::super::scenarios::{
    correspondence_denied_envelope, detail_preflight_bundle, retained_resolved,
};
use crate::harness::correspondence_history_certification::model::{
    CorrespondenceHistoryCertificationRejection, CorrespondenceHistoryFailureClass,
};

pub(crate) fn unsupported_correspondence_family_rejection(
) -> Result<CorrespondenceHistoryCertificationRejection, IdentityDerivationDenial> {
    let preflight = detail_preflight_bundle();
    let correspondence = resolve_correspondence_evidence(
        CorrespondenceEvaluationRequest::unsupported_structural_family(
            "unsupported_test_family",
            StructuralCandidateDiscoveryPlan::FingerprintBucketBounded,
            1,
        ),
    )
    .expect("unsupported structural family should resolve into denial");
    let envelope = compose_correspondence_historical_envelope(
        crate::execution::execute_preflight_bundle(&preflight).expect("execution should succeed"),
        correspondence,
        retained_resolved("basis:a")?,
    );
    let parity_bundle = build_correspondence_historical_parity_bundle(
        &envelope,
        Some(preflight.plan().query().validated_query_digest().clone()),
        Some(preflight.basis().proof().digest().clone()),
    )
    .expect("unsupported correspondence denial bundle should build");
    Ok(CorrespondenceHistoryCertificationRejection {
        failure_class: CorrespondenceHistoryFailureClass::CorrespondenceDenied,
        failure_digest: parity_bundle
            .failure_digest()
            .expect("unsupported correspondence denial should emit failure digest")
            .as_str()
            .to_string(),
        counter_snapshot_digest: Some(parity_bundle.counter_snapshot_digest().as_str().to_string()),
    })
}

pub(crate) fn broad_candidate_scan_rejection(
) -> Result<CorrespondenceHistoryCertificationRejection, IdentityDerivationDenial> {
    let error = crate::correspondence::CorrespondenceEvaluationError::BroadStructuralScanRequired;
    Ok(CorrespondenceHistoryCertificationRejection {
        failure_class: CorrespondenceHistoryFailureClass::CorrespondenceDenied,
        failure_digest: format!(
            "correspondence:{:?}:{}",
            error.failure_class(),
            error.reason()
        ),
        counter_snapshot_digest: Some(
            build_correspondence_historical_parity_bundle(
                &correspondence_denied_envelope()?,
                Some(
                    detail_preflight_bundle()
                        .plan()
                        .query()
                        .validated_query_digest()
                        .clone(),
                ),
                Some(detail_preflight_bundle().basis().proof().digest().clone()),
            )
            .expect("denied correspondence bundle should build")
            .counter_snapshot_digest()
            .as_str()
            .to_string(),
        ),
    })
}
