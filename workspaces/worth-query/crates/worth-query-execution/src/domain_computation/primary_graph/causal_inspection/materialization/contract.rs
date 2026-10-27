use worth_runtime_bridge::facade::{
    BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner, BridgeCausalExplanationEnvelope,
};

use super::policy::{
    CausalInspectionMaterializationError, CausalInspectionMaterializationErrorKind,
    CausalInspectionMaterializationPolicy,
};
use crate::domain_computation::evidence_identity::WorthQueryEvidenceIdentity;
use crate::domain_computation::evidence_identity::WorthQueryEvidenceTag;
use crate::domain_computation::primary_graph::causal_inspection::inventory::CausalEvidenceFamily;

pub(super) fn validate_materialization_contract(
    query_observation_identity: &WorthQueryEvidenceIdentity,
    requested_families: &[CausalEvidenceFamily],
    envelope: &BridgeCausalExplanationEnvelope,
    materialization_policy: CausalInspectionMaterializationPolicy,
) -> Result<(), CausalInspectionMaterializationError> {
    if envelope.bindings().is_empty() {
        return Err(CausalInspectionMaterializationError::new(
            CausalInspectionMaterializationErrorKind::MaterializationPolicyOverclaim,
            |identity| {
                identity
                    .field_shape(
                        WorthQueryEvidenceTag::new("policy"),
                        materialization_policy.as_str(),
                    )
                    .field_value(
                        WorthQueryEvidenceTag::new("envelope"),
                        envelope.envelope_for_reporting(),
                    )
                    .field_usize(WorthQueryEvidenceTag::new("bindings"), 0)
            },
        ));
    }
    validate_query_observation_binding(query_observation_identity, envelope)?;
    validate_requested_replay_posture(requested_families, envelope)
}

fn validate_query_observation_binding(
    query_observation_identity: &WorthQueryEvidenceIdentity,
    envelope: &BridgeCausalExplanationEnvelope,
) -> Result<(), CausalInspectionMaterializationError> {
    let mut query_observation_binding_count = 0;
    let mut query_observation_matches_subject = false;
    let expected_query_observation_identity = query_observation_identity.bridge_evidence_identity();
    for binding in envelope.bindings() {
        if binding.owner() != BridgeCausalEvidenceOwner::Query
            || binding.family() != BridgeCausalEvidenceFamily::QueryObservation
        {
            continue;
        }
        query_observation_binding_count += 1;
        query_observation_matches_subject |=
            binding.reference_evidence_identity() == expected_query_observation_identity;
    }

    match (
        query_observation_binding_count,
        query_observation_matches_subject,
    ) {
        (0, _) => Err(query_observation_binding_error(
            CausalInspectionMaterializationErrorKind::QueryObservationBindingMissing,
            query_observation_identity,
            envelope,
            0,
        )),
        (1, true) => Ok(()),
        (1, false) => Err(query_observation_binding_error(
            CausalInspectionMaterializationErrorKind::QueryObservationBindingMismatch,
            query_observation_identity,
            envelope,
            1,
        )),
        (binding_count, _) => Err(query_observation_binding_error(
            CausalInspectionMaterializationErrorKind::QueryObservationBindingOverclaim,
            query_observation_identity,
            envelope,
            binding_count,
        )),
    }
}

fn query_observation_binding_error(
    kind: CausalInspectionMaterializationErrorKind,
    query_observation_identity: &WorthQueryEvidenceIdentity,
    envelope: &BridgeCausalExplanationEnvelope,
    query_observation_binding_count: usize,
) -> CausalInspectionMaterializationError {
    CausalInspectionMaterializationError::new(kind, |identity| {
        identity
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("expected_query_observation"),
                query_observation_identity,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("query_observation_binding_count"),
                query_observation_binding_count,
            )
            .field_value(
                WorthQueryEvidenceTag::new("envelope"),
                envelope.envelope_for_reporting(),
            )
    })
}

fn validate_requested_replay_posture(
    requested_families: &[CausalEvidenceFamily],
    envelope: &BridgeCausalExplanationEnvelope,
) -> Result<(), CausalInspectionMaterializationError> {
    let bridge_replay_requested = requested_families.contains(&CausalEvidenceFamily::BridgeReplay);
    let signal_cursor_requested =
        requested_families.contains(&CausalEvidenceFamily::SignalReplayCursor);
    if !bridge_replay_requested && !signal_cursor_requested {
        return Ok(());
    }

    let bridge_replay_bound = envelope.bindings().iter().any(|binding| {
        binding.owner() == BridgeCausalEvidenceOwner::RuntimeBridge
            && matches!(
                binding.family(),
                BridgeCausalEvidenceFamily::BridgeStreamReplay
                    | BridgeCausalEvidenceFamily::BridgeWritebackReplay
            )
    });
    let signal_cursor_bound = envelope.bindings().iter().any(|binding| {
        binding.owner() == BridgeCausalEvidenceOwner::Signal
            && binding.family() == BridgeCausalEvidenceFamily::SignalReplayCursor
    });
    let bridge_replay_satisfied = !bridge_replay_requested || bridge_replay_bound;
    let signal_cursor_satisfied = !signal_cursor_requested || signal_cursor_bound;
    if bridge_replay_satisfied && signal_cursor_satisfied {
        return Ok(());
    }

    Err(CausalInspectionMaterializationError::new(
        CausalInspectionMaterializationErrorKind::ReplayPostureUnsupported,
        |identity| {
            identity
                .field_value(
                    WorthQueryEvidenceTag::new("envelope"),
                    envelope.envelope_for_reporting(),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("bridge_replay_requested"),
                    bridge_replay_requested,
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("bridge_replay_bound"),
                    bridge_replay_bound,
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("signal_cursor_requested"),
                    signal_cursor_requested,
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("signal_cursor_bound"),
                    signal_cursor_bound,
                )
        },
    ))
}
