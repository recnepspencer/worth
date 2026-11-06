use super::*;

pub(in crate::domain_capabilities::certification::reports) fn store_backed_replay_gap_request(
) -> WorthQueryLowerRuntimeExplanationRequest {
    let (reference_set, target) = replay_gap_inputs();
    WorthQueryLowerRuntimeExplanationRequest::explains_store_backed_replay_gap(
        reference_set,
        target,
        vec![CausalEvidenceFamily::QueryInspection],
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
}

pub(in crate::domain_capabilities::certification::reports) fn store_backed_replay_gap_parts(
) -> (CausalEvidenceReferenceSet, CausalInspectionTarget) {
    replay_gap_inputs()
}

fn replay_gap_inputs() -> (CausalEvidenceReferenceSet, CausalInspectionTarget) {
    let observation =
        QueryObservationReceipt::certification_historical_replay_fixture("domain-capability");
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
