use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

pub(super) fn authoritative_intent_receipt_identity_fixture(
    intent_name: &str,
    strategy_identity: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeIntentReceipt)
            .field_shape(WorthQueryEvidenceTag::new("intent_name"), intent_name)
            .field_shape(WorthQueryEvidenceTag::new("execution_kind"), "mutating")
            .field_value(
                WorthQueryEvidenceTag::new("strategy_identity"),
                strategy_identity,
            )
            .field_shape(WorthQueryEvidenceTag::new("strategy_version"), "1.0")
            .field_value(
                WorthQueryEvidenceTag::new("strategy_descriptor_digest"),
                "strategy-descriptor",
            )
            .field_value(
                WorthQueryEvidenceTag::new("canonical_input_digest"),
                "canonical-input",
            )
            .field_value(WorthQueryEvidenceTag::new("outcome_digest"), "outcome")
            .field_value_sequence(
                WorthQueryEvidenceTag::new("invariant_evidence"),
                ["invariant|one", "invariant:two"],
            )
            .field_shape(WorthQueryEvidenceTag::new("source_lane"), "strategy")
            .field_shape(WorthQueryEvidenceTag::new("target_lane"), "workspace")
            .optional_identity(
                WorthQueryEvidenceTag::new("effect_trigger_digest"),
                Option::<&str>::None,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("commit_evidence_identity"),
                &sample_identity(
                    WorthQueryEvidenceScope::WriteReceiptCommitIdentity,
                    "commit|identity",
                )?,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("snapshot_evidence_identity"),
                &sample_identity(
                    WorthQueryEvidenceScope::WriteReceiptSnapshotIdentity,
                    "snapshot|identity",
                )?,
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("affected_live_view_id"),
                ["live|view"],
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("affected_derived_view_id"),
                ["derived|view"],
            )
            .field_usize(
                WorthQueryEvidenceTag::new("considered_computed_view_count"),
                1,
            )
            .field_usize(WorthQueryEvidenceTag::new("considered_effect_count"), 1)
            .field_usize(WorthQueryEvidenceTag::new("delivered_effect_count"), 1)
            .field_usize(WorthQueryEvidenceTag::new("pending_write_intent_count"), 0)
            .field_usize(WorthQueryEvidenceTag::new("suppressed_effect_count"), 0)
            .field_usize(
                WorthQueryEvidenceTag::new("meaningful_effect_suppression_count"),
                0,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("effect_expression_failure_count"),
                0,
            )
            .field_bool(WorthQueryEvidenceTag::new("refresh_fallback"), false)
            .field_shape(
                WorthQueryEvidenceTag::new("admission_family"),
                "authoritative",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("covered_entrypoint"),
                "runtime.write",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("execution_seam"),
                "intent-authority",
            )
            .field_value(
                WorthQueryEvidenceTag::new("admission_decision_digest"),
                "decision",
            )
            .field_value(
                WorthQueryEvidenceTag::new("execution_handoff_digest"),
                "handoff",
            )
            .field_value(
                WorthQueryEvidenceTag::new("execution_binding_digest"),
                "binding",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("execution_provenance_chain_identity"),
                &sample_identity(
                    WorthQueryEvidenceScope::IntentExecutionProvenanceChain,
                    "provenance|chain",
                )?,
            )
            .field_value(
                WorthQueryEvidenceTag::new("decision_trace_digest"),
                "decision-trace",
            )
            .seal()?,
    )
}

pub(super) fn preview_intent_receipt_inspection_basis_fixture(
    intent_name: &str,
    basis_label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let basis_admission_identity =
        sample_identity(WorthQueryEvidenceScope::PreviewBasisAdmission, basis_label)?;
    let basis_evidence_identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewIntentBasisEvidence)
            .field_shape(WorthQueryEvidenceTag::new("intent_name"), intent_name)
            .field_usize(WorthQueryEvidenceTag::new("basis_evidence_count"), 2)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("basis_admission_identity"),
                &basis_admission_identity,
            )
            .seal()?;
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewIntentReceiptInspectionBasis)
            .field_shape(WorthQueryEvidenceTag::new("intent_name"), intent_name)
            .field_usize(WorthQueryEvidenceTag::new("basis_evidence_count"), 2)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("basis_evidence"),
                &basis_evidence_identity,
            )
            .seal()?,
    )
}

pub(super) fn preview_intent_admission_identity_fixture(
    intent_name: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewIntentAdmission)
            .field_shape(WorthQueryEvidenceTag::new("intent_name"), intent_name)
            .field_shape(WorthQueryEvidenceTag::new("strategy_identity"), "strategy")
            .field_shape(WorthQueryEvidenceTag::new("strategy_version"), "1.0")
            .field_value(
                WorthQueryEvidenceTag::new("canonical_input_digest"),
                "canonical-input",
            )
            .field_shape(WorthQueryEvidenceTag::new("source_lane"), "strategy")
            .field_shape(WorthQueryEvidenceTag::new("target_lane"), "preview")
            .field_shape(
                WorthQueryEvidenceTag::new("effect_policy"),
                "sandboxed-write-intent",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("admitted_action"),
                "write-intent",
            )
            .field_shape(WorthQueryEvidenceTag::new("admitted_lane"), "preview-truth")
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("basis_admission_identity"),
                &sample_identity(
                    WorthQueryEvidenceScope::PreviewBasisAdmission,
                    "preview-basis",
                )?,
            )
            .seal()?,
    )
}

pub(super) fn preview_intent_receipt_identity_fixture(
    admission_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewIntentReceipt)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("admission_identity"),
                admission_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("posture"),
                "preview-local-staged-no-authoritative-execution",
            )
            .seal()?,
    )
}

pub(super) fn preview_intent_receipt_inspection_identity_fixture(
    intent_name: &str,
    basis_identity: &WorthQueryEvidenceIdentity,
    admission_identity: &WorthQueryEvidenceIdentity,
    receipt_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewIntentReceiptInspection)
            .field_shape(WorthQueryEvidenceTag::new("intent_name"), intent_name)
            .field_value(WorthQueryEvidenceTag::new("strategy_identity"), "strategy")
            .field_shape(WorthQueryEvidenceTag::new("strategy_version"), "1.0")
            .field_value(
                WorthQueryEvidenceTag::new("canonical_input_digest"),
                "canonical-input",
            )
            .field_shape(WorthQueryEvidenceTag::new("source_lane"), "strategy")
            .field_shape(WorthQueryEvidenceTag::new("target_lane"), "preview")
            .field_shape(
                WorthQueryEvidenceTag::new("effect_policy"),
                "sandboxed-write-intent",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("basis_identity"), basis_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("admission_identity"),
                admission_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("receipt_identity"),
                receipt_identity,
            )
            .seal()?,
    )
}

pub(super) fn sample_identity(
    scope: WorthQueryEvidenceScope,
    label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(scope)
        .field_shape(WorthQueryEvidenceTag::new("fixture"), label)
        .seal()?)
}
