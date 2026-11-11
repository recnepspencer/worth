use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::projection_consumption::ProjectionAuthorityOutcome;
use crate::runtime::tests::support::*;
use crate::runtime::{
    WorthQueryConcurrentHostileMatrixCounterSnapshot, WorthQueryConcurrentHostileMatrixTopology,
};
use worth_foundational::facade::{AspectValue, InternedString};

pub(super) fn consume_artifact_title(
    artifact: &WorthQueryPublishedDerivedArtifactHandle,
) -> String {
    match hostile_consume_title_attempt(artifact) {
        WorthQueryPublishedProjectionAuthorityOutcome::Current(
            ProjectionAuthorityOutcome::Admitted(completed),
        ) => completed
            .facts()
            .display_fields()
            .first()
            .and_then(|fact| match fact.native_value().scalar()? {
                AspectValue::String(InternedString::Stored(value)) => {
                    let value = value.as_string();
                    Some(value.as_str())
                }
                AspectValue::String(InternedString::Raw(value)) => Some(value.as_str()),
                AspectValue::String(InternedString::Symbol(_)) => None,
                _ => None,
            })
            .unwrap_or("none")
            .to_string(),
        WorthQueryPublishedProjectionAuthorityOutcome::ResultState(state) => {
            state.result_state_for_reporting().to_string()
        }
        other => panic!("unexpected projection consumption posture: {other:?}"),
    }
}

pub(super) fn raw_matrix_digest(
    topology: WorthQueryConcurrentHostileMatrixTopology,
    receipt_digests: &[String],
    reader_result_digests: &[String],
    published_artifact_digests: &[String],
    preview_closeout_digests: &[String],
    branch_basis_digests: &[String],
    replay_digest: &str,
    counters: &WorthQueryConcurrentHostileMatrixCounterSnapshot,
) -> String {
    WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact,
    )
    .field_usize(
        WorthQueryEvidenceTag::new("reader_thread_count"),
        topology.reader_thread_count(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("submitter_thread_count"),
        topology.submitter_thread_count(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("submission_round_count"),
        topology.submission_round_count(),
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("receipt_digest"),
        receipt_digests.iter().map(String::as_str),
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("reader_result_digest"),
        reader_result_digests.iter().map(String::as_str),
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("published_artifact_digest"),
        published_artifact_digests.iter().map(String::as_str),
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("preview_closeout_digest"),
        preview_closeout_digests.iter().map(String::as_str),
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("branch_basis_digest"),
        branch_basis_digests.iter().map(String::as_str),
    )
    .field_value(WorthQueryEvidenceTag::new("replay_digest"), replay_digest)
    .field_usize(
        WorthQueryEvidenceTag::new("counter_residue_count"),
        counters.exact_zero_residue_count(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("registry_lease_count"),
        counters.published_artifact_registry_lease_count(),
    )
    .seal()
    .expect("fixture digest must fit")
    .terminal_projection_for_reporting()
    .to_string()
}
