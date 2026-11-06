use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

pub(in crate::domain_capabilities::certification::reports) fn store_backed_replay_gap_request(
) -> Result<WorthQueryLowerRuntimeExplanationRequest, IdentityDerivationDenial> {
    let (reference_set, target) = replay_gap_inputs()?;
    Ok(
        WorthQueryLowerRuntimeExplanationRequest::explains_store_backed_replay_gap(
            reference_set,
            target,
            vec![CausalEvidenceFamily::QueryInspection],
            CausalInspectionRedactionPolicy::PreserveDetail,
            CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
        ),
    )
}

pub(in crate::domain_capabilities::certification::reports) fn store_backed_replay_gap_parts(
) -> Result<(CausalEvidenceReferenceSet, CausalInspectionTarget), IdentityDerivationDenial> {
    Ok(replay_gap_inputs()?)
}

fn replay_gap_inputs(
) -> Result<(CausalEvidenceReferenceSet, CausalInspectionTarget), IdentityDerivationDenial> {
    let observation =
        QueryObservationReceipt::certification_historical_replay_fixture("domain-capability")?;
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

    Ok((reference_set, target))
}
