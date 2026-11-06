use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::super::WorthQueryFeedbackPhaseGraphInspection;
use super::declaration::WorthQueryEffectExpressionFailurePosture;
use super::delivery::{WorthQueryEffectDelivery, WorthQueryEffectDeliveryFamily};
use super::phase::WorthQueryEffectPhaseEvidence;
use super::registry::WorthQueryEffectRuntime;
use crate::runtime::WorthQueryAspectTouch;

pub(super) struct EffectInspectionDigestSet {
    pub(super) trigger_digest: String,
    pub(super) condition_digest: String,
    pub(super) declaration_digest: String,
    pub(super) pending_delivery_digest: String,
    pub(super) latest_phase_digest: Option<String>,
    pub(super) inspection_digest: String,
}

pub(super) fn effect_inspection_digests(
    effect: &WorthQueryEffectRuntime,
    condition_descriptor: &str,
    condition_inputs: &[WorthQueryAspectTouch],
    condition_outputs: &[WorthQueryAspectTouch],
    condition_failure_posture: Option<WorthQueryEffectExpressionFailurePosture>,
    pending_delivery_count: usize,
    pending_delivered_count: usize,
    pending_suppressed_count: usize,
    pending_expression_failure_count: usize,
    pending_write_intent_count: usize,
    latest_delivery_family: Option<&WorthQueryEffectDeliveryFamily>,
    latest_phase_evidence: Option<&WorthQueryEffectPhaseEvidence>,
    feedback_graph: Option<&WorthQueryFeedbackPhaseGraphInspection>,
) -> Result<EffectInspectionDigestSet, IdentityDerivationDenial> {
    let trigger_identity = trigger_inspection_identity(effect)?;
    let condition_identity = condition_inspection_identity(
        effect,
        condition_descriptor,
        condition_inputs,
        condition_outputs,
        condition_failure_posture,
    )?;
    let declaration_identity =
        declaration_inspection_identity(effect, &trigger_identity, &condition_identity)?;
    let pending_delivery_identity = pending_delivery_inspection_identity(
        effect,
        pending_delivery_count,
        pending_delivered_count,
        pending_suppressed_count,
        pending_expression_failure_count,
        pending_write_intent_count,
    )?;
    let latest_phase_identity = latest_phase_evidence.map(effect_phase_inspection_identity);
    let inspection_identity = inspection_identity(
        latest_delivery_family,
        feedback_graph,
        &declaration_identity,
        &pending_delivery_identity,
        latest_phase_identity.as_ref()?,
    )?;

    Ok(EffectInspectionDigestSet {
        trigger_digest: trigger_identity.as_str().to_string(),
        condition_digest: condition_identity.as_str().to_string(),
        declaration_digest: declaration_identity.as_str().to_string(),
        pending_delivery_digest: pending_delivery_identity.as_str().to_string(),
        latest_phase_digest: latest_phase_identity
            .as_ref()
            .map(|identity| identity.as_str().to_string()),
        inspection_digest: inspection_identity.as_str().to_string(),
    })
}

fn trigger_inspection_identity(
    effect: &WorthQueryEffectRuntime,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceiptInspection)
            .field_shape(
                WorthQueryEvidenceTag::new("artifact_kind"),
                "effect-trigger-inspection",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("name"),
                effect.declaration.name(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("source_kind"),
                effect.declaration.trigger().source_kind().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("source"),
                effect.declaration.trigger().source_name(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("write_adjacent_trigger"),
                effect.declaration.write_adjacent_trigger().identity(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("aspects"),
                effect
                    .declaration
                    .trigger()
                    .aspect_touches()
                    .iter()
                    .map(WorthQueryAspectTouch::admitted_touch_digest_part),
            )
            .seal()?,
    )
}

fn condition_inspection_identity(
    effect: &WorthQueryEffectRuntime,
    condition_descriptor: &str,
    condition_inputs: &[WorthQueryAspectTouch],
    condition_outputs: &[WorthQueryAspectTouch],
    condition_failure_posture: Option<WorthQueryEffectExpressionFailurePosture>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceiptInspection)
            .field_shape(
                WorthQueryEvidenceTag::new("artifact_kind"),
                "effect-condition-inspection",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("name"),
                effect.declaration.name(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("descriptor"),
                condition_descriptor,
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("inputs"),
                condition_inputs
                    .iter()
                    .map(WorthQueryAspectTouch::admitted_touch_digest_part),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("outputs"),
                condition_outputs
                    .iter()
                    .map(WorthQueryAspectTouch::admitted_touch_digest_part),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("failure_posture"),
                effect_condition_failure_posture_label(condition_failure_posture),
            )
            .seal()?,
    )
}

fn declaration_inspection_identity(
    effect: &WorthQueryEffectRuntime,
    trigger_identity: &WorthQueryEvidenceIdentity,
    condition_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceiptInspection)
            .field_shape(
                WorthQueryEvidenceTag::new("artifact_kind"),
                "effect-declaration-inspection",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("name"),
                effect.declaration.name(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("trigger"), trigger_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("condition"), condition_identity)
            .field_shape(
                WorthQueryEvidenceTag::new("action"),
                effect.declaration.action().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("target_lane"),
                effect.declaration.target_lane().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("target"),
                effect.declaration.target(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("policy"),
                effect.declaration.effect_policy().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("suppression"),
                effect.declaration.suppression_policy().as_str(),
            )
            .seal()?,
    )
}

fn pending_delivery_row_identity(
    delivery: &WorthQueryEffectDelivery,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceiptPhase)
            .field_shape(
                WorthQueryEvidenceTag::new("effect_name"),
                delivery.effect_name(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("trigger_commit"),
                delivery.trigger_commit_evidence_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("trigger_source_kind"),
                delivery.trigger_source_kind().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                effect_delivery_family_label(delivery.family()),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("admitted_aspect_touch"),
                delivery
                    .aspect_touches()
                    .iter()
                    .map(WorthQueryAspectTouch::admitted_touch_digest_part),
            )
            .seal()?,
    )
}

fn pending_delivery_inspection_identity(
    effect: &WorthQueryEffectRuntime,
    pending_delivery_count: usize,
    pending_delivered_count: usize,
    pending_suppressed_count: usize,
    pending_expression_failure_count: usize,
    pending_write_intent_count: usize,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let delivery_identities = effect
        .deliveries
        .iter()
        .map(pending_delivery_row_identity)
        .collect::<Vec<_>>();

    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceiptPhase)
            .field_shape(
                WorthQueryEvidenceTag::new("artifact_kind"),
                "pending-delivery",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("name"),
                effect.declaration.name(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("pending_delivery_count"),
                pending_delivery_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("pending_delivered_count"),
                pending_delivered_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("pending_suppressed_count"),
                pending_suppressed_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("pending_expression_failure_count"),
                pending_expression_failure_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("pending_write_intent_count"),
                pending_write_intent_count,
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("deliveries"),
                delivery_identities.iter(),
            )
            .seal()?,
    )
}

fn inspection_identity(
    latest_delivery_family: Option<&WorthQueryEffectDeliveryFamily>,
    feedback_graph: Option<&WorthQueryFeedbackPhaseGraphInspection>,
    declaration_identity: &WorthQueryEvidenceIdentity,
    pending_delivery_identity: &WorthQueryEvidenceIdentity,
    latest_phase_identity: Option<&WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceiptInspection)
            .field_shape(
                WorthQueryEvidenceTag::new("artifact_kind"),
                "effect-inspection",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("declaration"),
                declaration_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("pending_delivery"),
                pending_delivery_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("latest_family"),
                latest_delivery_family
                    .map(effect_delivery_family_label)
                    .unwrap_or("none"),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("latest_phase"),
                latest_phase_identity,
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("feedback_graph"),
                feedback_graph.map(|graph| graph.graph_identity()),
            )
            .seal()?,
    )
}

fn effect_condition_failure_posture_label(
    posture: Option<WorthQueryEffectExpressionFailurePosture>,
) -> &'static str {
    match posture {
        Some(WorthQueryEffectExpressionFailurePosture::Admitted) => "admitted",
        Some(WorthQueryEffectExpressionFailurePosture::DeterministicFailure) => {
            "deterministic-failure"
        }
        None => "none",
    }
}

fn effect_delivery_family_label(family: &WorthQueryEffectDeliveryFamily) -> &'static str {
    match family {
        WorthQueryEffectDeliveryFamily::Delivered => "delivered",
        WorthQueryEffectDeliveryFamily::PendingWriteIntent => "pending-write-intent",
        WorthQueryEffectDeliveryFamily::Suppressed => "suppressed",
        WorthQueryEffectDeliveryFamily::ExpressionFailed => "expression-failed",
    }
}

fn effect_phase_inspection_identity(
    phase: &WorthQueryEffectPhaseEvidence,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceiptInspection)
            .field_shape(
                WorthQueryEvidenceTag::new("artifact_kind"),
                "effect-phase-inspection",
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("phases"),
                phase.phases().iter().map(|entry| entry.as_str()),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("loop_prevention"),
                phase.loop_prevention().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("idempotence"),
                phase.idempotence().as_str(),
            )
            .seal()?,
    )
}
