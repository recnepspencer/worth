use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub(in crate::subscription) fn diagnostic_counters_identity(
    diagnostic_trace_emission_count: u64,
    diagnostic_bundle_emission_count: u64,
    denied_bundle_emission_count: u64,
    diagnostic_missing_stage_denial_count: u64,
    diagnostic_bundle_width: u64,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_diagnostic_counters_v1",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("diagnostic_trace_emission"),
                diagnostic_trace_emission_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("diagnostic_bundle_emission"),
                diagnostic_bundle_emission_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("diagnostic_denied_bundle_emission"),
                denied_bundle_emission_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("diagnostic_missing_stage_denial"),
                diagnostic_missing_stage_denial_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("diagnostic_bundle_width"),
                diagnostic_bundle_width as usize,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn diagnostic_bundle_width_identity(
    stage_evidence_count: usize,
    failure_evidence_count: usize,
    hostile_row_reference_count: usize,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_diagnostic_bundle_width_v1",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("stage_evidence_count"),
                stage_evidence_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("failure_evidence_count"),
                failure_evidence_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("hostile_row_reference_count"),
                hostile_row_reference_count,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn diagnostic_assembly_receipt_identity(
    bundle_assembly_posture: &str,
    stage_evidence_composition_count: usize,
    semantic_label_carry_forward_count: usize,
    stage_rederivation_count: usize,
    bundle_width_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_diagnostic_assembly_receipt_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("bundle_assembly_posture"),
            bundle_assembly_posture,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("stage_evidence_composition_count"),
            stage_evidence_composition_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("semantic_label_carry_forward_count"),
            semantic_label_carry_forward_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("stage_rederivation_count"),
            stage_rederivation_count,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("bundle_width"),
            bundle_width_identity,
        )
        .seal()
}

pub(in crate::subscription) fn diagnostic_semantic_labels_identity(
    query_family_label: &str,
    declaration_family_label: &str,
    bridge_family_label: &str,
    bridge_slice_labels: &[String],
    basis_posture_label: &str,
    signal_strategy_class_label: &str,
    live_graph_access_posture_label: &str,
    support_posture_label: &str,
    denial_or_coverage_class_label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_diagnostic_semantic_labels_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("query_family"),
            query_family_label,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("declaration_family"),
            declaration_family_label,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("bridge_family"),
            bridge_family_label,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("basis_posture"),
            basis_posture_label,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("signal_strategy_class"),
            signal_strategy_class_label,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("live_graph_access_posture"),
            live_graph_access_posture_label,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("support_posture"),
            support_posture_label,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("denial_or_coverage_class"),
            denial_or_coverage_class_label,
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("bridge_slices"),
            bridge_slice_labels.iter().map(String::as_str),
        )
        .seal()
}

pub(in crate::subscription) fn diagnostic_failure_identity(
    failure_kind: &str,
    diagnostic_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_diagnostic_failure_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("failure_kind"), failure_kind)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("diagnostic"),
            diagnostic_identity,
        )
        .seal()
}

pub(in crate::subscription) fn diagnostic_admitted_bundle_identity(
    trace_identity: &WorthQueryEvidenceIdentity,
    labels_identity: &WorthQueryEvidenceIdentity,
    support_report_identity: &WorthQueryEvidenceIdentity,
    lifecycle_identity: &WorthQueryEvidenceIdentity,
    receipt_identity: &WorthQueryEvidenceIdentity,
    counters_identity: &WorthQueryEvidenceIdentity,
    admission_identity: &WorthQueryEvidenceIdentity,
    continuation_identity: Option<&WorthQueryEvidenceIdentity>,
    preview_identity: Option<&WorthQueryEvidenceIdentity>,
    closeout_identity: Option<&WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut composer = WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_admitted_diagnostic_bundle_v1",
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("trace"), trace_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("labels"), labels_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("support"),
            support_report_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("lifecycle_certification"),
            lifecycle_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("receipt"), receipt_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counters_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("admission"), admission_identity);
    if let Some(continuation_identity) = continuation_identity {
        composer = composer.field_evidence_identity(
            WorthQueryEvidenceTag::new("continuation"),
            continuation_identity,
        );
    }
    if let Some(preview_identity) = preview_identity {
        composer = composer
            .field_evidence_identity(WorthQueryEvidenceTag::new("preview"), preview_identity);
    }
    if let Some(closeout_identity) = closeout_identity {
        composer = composer
            .field_evidence_identity(WorthQueryEvidenceTag::new("closeout"), closeout_identity);
    }
    composer.seal()
}

pub(in crate::subscription) fn diagnostic_denied_bundle_identity(
    trace_identity: &WorthQueryEvidenceIdentity,
    labels_identity: &WorthQueryEvidenceIdentity,
    failure_identity: &WorthQueryEvidenceIdentity,
    receipt_identity: &WorthQueryEvidenceIdentity,
    counters_identity: &WorthQueryEvidenceIdentity,
    support_report_identity: Option<&WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut composer = WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_denied_diagnostic_bundle_v1",
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("trace"), trace_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("labels"), labels_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("failure"), failure_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("receipt"), receipt_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counters_identity);
    if let Some(support_report_identity) = support_report_identity {
        composer = composer.field_evidence_identity(
            WorthQueryEvidenceTag::new("support"),
            support_report_identity,
        );
    }
    composer.seal()
}
