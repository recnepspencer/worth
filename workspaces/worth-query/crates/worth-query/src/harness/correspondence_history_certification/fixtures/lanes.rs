use crate::correspondence::{
    resolve_correspondence_evidence, CorrespondenceEvaluationRequest,
    StructuralCandidateDiscoveryPlan,
};
use crate::correspondence_history::{
    compose_correspondence_historical_envelope, CorrespondenceHistoricalEnvelope,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::facade::foundation::{
    admit_historical_evaluation_path, build_correspondence_historical_parity_bundle,
    resolve_historical_materialization_path, HistoricalCapabilityDescriptor,
    HistoricalEvaluationRequest, HistoricalMaterializationDescriptor,
    HistoricalPathReuseDescriptor, ResolvedHistoricalPathClass,
};

use super::scenarios::{
    ambiguity_envelope, detail_preflight_bundle, disagreement_envelope,
    lineage_authoritative_envelope, reconstruction_path_envelope, replay_path_envelope,
    retained_path_envelope, structural_unique_replay_envelope,
};
use crate::harness::correspondence_history_certification::model::CorrespondenceHistoryCertificationLane;

#[derive(Clone)]
pub(crate) struct CertificationLanes {
    pub(crate) lineage: CorrespondenceHistoryCertificationLane,
    pub(crate) structural: CorrespondenceHistoryCertificationLane,
    pub(crate) disagreement: CorrespondenceHistoryCertificationLane,
    pub(crate) ambiguity: CorrespondenceHistoryCertificationLane,
    pub(crate) retained: CorrespondenceHistoryCertificationLane,
    pub(crate) replay: CorrespondenceHistoryCertificationLane,
    pub(crate) reconstruction: CorrespondenceHistoryCertificationLane,
    pub(crate) drift: CorrespondenceHistoryCertificationLane,
}

pub(crate) fn lineage_authoritative_lane(
) -> Result<CorrespondenceHistoryCertificationLane, IdentityDerivationDenial> {
    Ok(lane_from_supported_envelope(
        lineage_authoritative_envelope()?,
    ))
}

pub(crate) fn structural_unique_replay_lane(
) -> Result<CorrespondenceHistoryCertificationLane, IdentityDerivationDenial> {
    Ok(lane_from_supported_envelope(
        structural_unique_replay_envelope()?,
    ))
}

pub(crate) fn disagreement_lane(
) -> Result<CorrespondenceHistoryCertificationLane, IdentityDerivationDenial> {
    Ok(lane_from_supported_envelope(disagreement_envelope()?))
}

pub(crate) fn ambiguity_lane(
) -> Result<CorrespondenceHistoryCertificationLane, IdentityDerivationDenial> {
    Ok(lane_from_supported_envelope(ambiguity_envelope()?))
}

pub(crate) fn retained_lane(
) -> Result<CorrespondenceHistoryCertificationLane, IdentityDerivationDenial> {
    Ok(lane_from_supported_envelope(retained_path_envelope()?))
}

pub(crate) fn replay_lane(
) -> Result<CorrespondenceHistoryCertificationLane, IdentityDerivationDenial> {
    Ok(lane_from_supported_envelope(replay_path_envelope()?))
}

pub(crate) fn reconstruction_lane(
) -> Result<CorrespondenceHistoryCertificationLane, IdentityDerivationDenial> {
    Ok(lane_from_supported_envelope(reconstruction_path_envelope()?))
}

pub(crate) fn prediction_drift_lane(
) -> Result<CorrespondenceHistoryCertificationLane, IdentityDerivationDenial> {
    let execution = crate::execution::execute_preflight_bundle(&detail_preflight_bundle())
        .expect("execution should succeed");
    let correspondence =
        resolve_correspondence_evidence(CorrespondenceEvaluationRequest::lineage_only(
            "subject:drift",
            "record:drift",
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            1,
        ))
        .expect("correspondence should resolve");
    let request = HistoricalEvaluationRequest::delta_replay_for_test(
        "basis:drift",
        1,
        8,
        HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
    )?;
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        "basis:drift",
        Some(crate::historical::AdmittedHistoricalPathClass::AdmittedDeltaReplayPath),
        true,
        false,
        false,
        true,
        HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
    )?;
    let admission =
        admit_historical_evaluation_path(request, capability).expect("admission should succeed");
    let historical = resolve_historical_materialization_path(
        admission,
        HistoricalMaterializationDescriptor::new_for_test(
            "basis:drift",
            ResolvedHistoricalPathClass::ResolvedDeltaReplayPath,
        )?
        .with_realized_work(3, 0),
    )
    .expect("historical drift path should resolve");

    Ok(lane_from_supported_envelope(
        compose_correspondence_historical_envelope(execution, correspondence, historical),
    ))
}

fn lane_from_supported_envelope(
    envelope: CorrespondenceHistoricalEnvelope,
) -> CorrespondenceHistoryCertificationLane {
    CorrespondenceHistoryCertificationLane {
        parity_bundle: build_correspondence_historical_parity_bundle(&envelope, None, None)
            .expect("supported envelope parity bundle should build"),
    }
}
