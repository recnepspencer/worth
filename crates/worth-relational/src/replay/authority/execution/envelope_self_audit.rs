use crate::history::data::HistoryDriftClass;
use crate::lineage::data::PublishedLineageArtifact;
use crate::replay::data::{
    CanonicalCommitEnvelope, ReplayMismatch, ReplayMismatchClass, ReplayObservableSurface,
    ReplayVerificationLayer,
};
use crate::runtime::RelationalRuntime;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;

use super::super::SelectedPublishedLineageAuthority;

pub(super) fn audit_retained_envelope_authority(
    runtime: &RelationalRuntime,
    mismatches: &mut Vec<ReplayMismatch>,
    envelope: &CanonicalCommitEnvelope,
    selected_lineage_authority: Option<&SelectedPublishedLineageAuthority>,
) {
    audit_schema_lineage_summary(runtime, mismatches, envelope);
    if envelope.has_lineage_authority() {
        audit_published_lineage_digest_basis(runtime, mismatches, envelope.published_lineage());
    }
    // A durably indexed artifact is a second authority to audit; the retained
    // envelope's own artifact was already audited above.
    if let Some(selected_lineage_authority) =
        selected_lineage_authority.filter(|selected| selected.indexed_source.is_some())
    {
        audit_published_lineage_digest_basis(
            runtime,
            mismatches,
            &selected_lineage_authority.artifact,
        );
    }
}

fn audit_schema_lineage_summary(
    runtime: &RelationalRuntime,
    mismatches: &mut Vec<ReplayMismatch>,
    envelope: &CanonicalCommitEnvelope,
) {
    let Some(transition) = envelope.schema_transition.as_ref() else {
        return;
    };
    let Some(reconciliation) = envelope.schema_reconciliation_descriptor.as_ref() else {
        return;
    };
    let lineage = &reconciliation.resulting_lineage;
    let expected_parent_schema_ids = vec![transition.source_schema_id.clone()];
    let expected_parent_schema_version_ids = vec![transition.source_schema_version_id];
    let lineage_matches_transition_basis = lineage.resulting_schema_id
        == transition.target_schema_id
        && lineage.resulting_schema_version_id == transition.target_schema_version_id
        && lineage.parent_schema_ids == expected_parent_schema_ids
        && lineage.parent_schema_version_ids == expected_parent_schema_version_ids;

    if lineage_matches_transition_basis {
        return;
    }

    runtime
        .performance_access()
        .count_replay_verification_layer(ReplayVerificationLayer::SummaryParity);
    mismatches.push(ReplayMismatch {
        class: ReplayMismatchClass::SchemaLineageDrift,
        history_drift_class: None,
        surface: ReplayObservableSurface::History,
        verification_layer: ReplayVerificationLayer::SummaryParity,
        detail: "retained schema lineage summary does not match transition source/target basis"
            .to_string(),
        expected: Some(format!(
            "target={:?}/{:?}|parents={:?}/{:?}",
            transition.target_schema_id,
            transition.target_schema_version_id,
            expected_parent_schema_ids,
            expected_parent_schema_version_ids
        )),
        observed: Some(format!("{lineage:?}")),
    });
}

fn audit_published_lineage_digest_basis(
    runtime: &RelationalRuntime,
    mismatches: &mut Vec<ReplayMismatch>,
    published_lineage: &PublishedLineageArtifact,
) {
    audit_lineage_event_batch_digest_basis(runtime, mismatches, published_lineage);
    audit_lineage_decision_log_digest_basis(runtime, mismatches, published_lineage);
}

fn audit_lineage_event_batch_digest_basis(
    runtime: &RelationalRuntime,
    mismatches: &mut Vec<ReplayMismatch>,
    published_lineage: &PublishedLineageArtifact,
) {
    let expected = published_lineage.event_batch_digest_basis();
    let observed = published_lineage.observed_event_batch_digest_basis();
    if expected == &observed {
        return;
    }

    push_lineage_digest_basis_mismatch(
        runtime,
        mismatches,
        "retained lineage event-batch digest basis does not match observed event content",
        format!("{expected:?}"),
        format!("{observed:?}"),
    );
}

fn audit_lineage_decision_log_digest_basis(
    runtime: &RelationalRuntime,
    mismatches: &mut Vec<ReplayMismatch>,
    published_lineage: &PublishedLineageArtifact,
) {
    let expected = published_lineage.decision_log_digest_basis();
    let observed = published_lineage.observed_decision_log_digest_basis();
    if expected == &observed {
        return;
    }

    push_lineage_digest_basis_mismatch(
        runtime,
        mismatches,
        "retained lineage decision-log digest basis does not match observed decision content",
        format!("{expected:?}"),
        format!("{observed:?}"),
    );
}

fn push_lineage_digest_basis_mismatch(
    runtime: &RelationalRuntime,
    mismatches: &mut Vec<ReplayMismatch>,
    detail: &str,
    expected: String,
    observed: String,
) {
    runtime
        .performance_access()
        .count_replay_verification_layer(ReplayVerificationLayer::DigestParity);
    mismatches.push(ReplayMismatch {
        class: ReplayMismatchClass::LineageDrift,
        history_drift_class: Some(HistoryDriftClass::ReplayAuthorityDrift),
        surface: ReplayObservableSurface::Lineage,
        verification_layer: ReplayVerificationLayer::DigestParity,
        detail: detail.to_string(),
        expected: Some(expected),
        observed: Some(observed),
    });
}
