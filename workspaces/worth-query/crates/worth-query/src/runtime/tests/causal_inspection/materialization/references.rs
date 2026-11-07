use worth_runtime_bridge::facade::{
    BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner, BridgeCausalEvidenceReference,
    BridgeCausalEvidenceReferenceIdentity, BridgeRouteIdentity,
};

use super::super::super::super::*;

pub(in crate::runtime::tests::causal_inspection) fn bridge_reference(
    identity: BridgeCausalEvidenceReferenceIdentity,
) -> BridgeCausalEvidenceReference {
    let family = identity.family();
    BridgeCausalEvidenceReference::new(BridgeCausalEvidenceOwner::RuntimeBridge, family, identity)
        .expect("bridge causal reference should be valid")
}

pub(in crate::runtime::tests::causal_inspection) fn external_reference(
    owner: BridgeCausalEvidenceOwner,
    identity: BridgeCausalEvidenceReferenceIdentity,
) -> BridgeCausalEvidenceReference {
    BridgeCausalEvidenceReference::new(owner, identity.family(), identity)
        .expect("external causal reference should be valid")
}

pub(in crate::runtime::tests::causal_inspection) fn query_reference(
    identity: BridgeCausalEvidenceReferenceIdentity,
) -> BridgeCausalEvidenceReference {
    BridgeCausalEvidenceReference::new(
        BridgeCausalEvidenceOwner::Query,
        BridgeCausalEvidenceFamily::QueryObservation,
        identity,
    )
    .expect("query causal reference should be valid")
}

pub(in crate::runtime::tests::causal_inspection) fn changed_reference_set(
    route_identity: &BridgeRouteIdentity,
) -> CausalEvidenceReferenceSet {
    let anchor = anchor_causal_observation(
        QueryObservationReceipt::fixture(
            CausalObservationOutcome::Changed,
            vec![
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::QueryInspection,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "query-inspection:phase5",
                    ),
                )
                .expect("fixture digest must fit"),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::BridgeRoute,
                    route_identity.bridge_admission_evidence(),
                )
                .expect("fixture digest must fit"),
            ],
        ),
        CausalInspectionReason::ChangedResult,
    )
    .expect("changed receipt should anchor");
    let CausalEvidenceReferenceResolution::Resolved { reference_set, .. } =
        resolve_causal_evidence_references(
            anchor,
            &[
                CausalEvidenceFamily::QueryInspection,
                CausalEvidenceFamily::BridgeRoute,
            ],
        )
        .expect("fixture digest must fit")
    else {
        panic!("changed references should resolve");
    };
    reference_set
}

pub(in crate::runtime::tests::causal_inspection) fn replay_reference_set_with_signal_cursor(
    route_identity: &BridgeRouteIdentity,
    signal_replay_cursor_identity: &str,
) -> CausalEvidenceReferenceSet {
    let anchor = anchor_causal_observation(
        QueryObservationReceipt::fixture(
            CausalObservationOutcome::Replayed,
            vec![
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::QueryInspection,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "query-inspection:replay-materialization",
                    ),
                )
                .expect("fixture digest must fit"),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::BridgeRoute,
                    route_identity.bridge_admission_evidence(),
                )
                .expect("fixture digest must fit"),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::SignalReplayCursor,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        signal_replay_cursor_identity,
                    ),
                )
                .expect("fixture digest must fit"),
            ],
        ),
        CausalInspectionReason::HistoricalReplayResult,
    )
    .expect("replay receipt should anchor");
    let CausalEvidenceReferenceResolution::Resolved { reference_set, .. } =
        resolve_causal_evidence_references(
            anchor,
            &[
                CausalEvidenceFamily::QueryInspection,
                CausalEvidenceFamily::BridgeRoute,
                CausalEvidenceFamily::SignalReplayCursor,
            ],
        )
        .expect("fixture digest must fit")
    else {
        panic!("replay references should resolve");
    };
    reference_set
}

pub(in crate::runtime::tests::causal_inspection) fn request_for(
    reference_set: CausalEvidenceReferenceSet,
    richness: CausalInspectionRichness,
) -> CausalInspectionRequest {
    let receipt = reference_set.anchor().observation_receipt();
    let target = causal_inspection_target(
        receipt.observation_target().clone(),
        receipt.result_shape_context().clone(),
    )
    .expect("target should match receipt");
    request_causal_inspection(
        reference_set,
        target,
        CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation,
        richness,
        &[CausalEvidenceFamily::BridgeRoute],
    )
    .expect("causal inspection request should be admitted to admission boundary")
}

pub(in crate::runtime::tests::causal_inspection) fn request_for_families(
    reference_set: CausalEvidenceReferenceSet,
    richness: CausalInspectionRichness,
    requested_evidence_families: &[CausalEvidenceFamily],
) -> CausalInspectionRequest {
    let receipt = reference_set.anchor().observation_receipt();
    let target = causal_inspection_target(
        receipt.observation_target().clone(),
        receipt.result_shape_context().clone(),
    )
    .expect("target should match receipt");
    request_causal_inspection(
        reference_set,
        target,
        CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation,
        richness,
        requested_evidence_families,
    )
    .expect("causal inspection request should be admitted to admission boundary")
}
