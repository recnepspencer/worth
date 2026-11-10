use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub(in crate::subscription) fn diagnostic_evidence_identity(
    stage: &str,
    outcome: &str,
    reason: &str,
    source_identity: &WorthQueryEvidenceIdentity,
    counter_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_diagnostic_evidence_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("stage"), stage)
        .field_shape(WorthQueryEvidenceTag::new("outcome"), outcome)
        .field_shape(WorthQueryEvidenceTag::new("reason"), reason)
        .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counter_identity)
        .seal()
}

pub(in crate::subscription) fn diagnostic_stage_trace_identity(
    stage: &str,
    outcome: &str,
    source_identity: &WorthQueryEvidenceIdentity,
    evidence_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_diagnostic_stage_trace_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("stage"), stage)
        .field_shape(WorthQueryEvidenceTag::new("outcome"), outcome)
        .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("evidence"), evidence_identity)
        .seal()
}

pub(in crate::subscription) fn diagnostic_trace_identity<'a>(
    terminal_stage: &str,
    counters_identity: &WorthQueryEvidenceIdentity,
    stage_traces: impl IntoIterator<Item = &'a WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_diagnostic_trace_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("terminal_stage"), terminal_stage)
        .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counters_identity)
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("stages"), stage_traces)
        .seal()
}

pub(in crate::subscription) fn diagnostic_selection_context_selected_identity(
    equivalence_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_diagnostic_selection_context_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), "selected")
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("equivalence"),
                equivalence_identity,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn diagnostic_selection_context_denied_identity(
    source_identity: &WorthQueryEvidenceIdentity,
    query_family_label: &str,
    declaration_family_label: &str,
    basis_posture_label: &str,
    live_graph_access_posture_label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_diagnostic_selection_context_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("kind"), "selection_denied")
        .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
        .field_shape(
            WorthQueryEvidenceTag::new("query_family"),
            query_family_label,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("declaration_family"),
            declaration_family_label,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("basis_posture"),
            basis_posture_label,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("live_graph_access_posture"),
            live_graph_access_posture_label,
        )
        .seal()
}
