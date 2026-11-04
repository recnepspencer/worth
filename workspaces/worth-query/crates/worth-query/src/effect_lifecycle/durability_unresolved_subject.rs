//! Query preserves the subject of every unresolved Native publication.
//! Initialization, canonical commits and branch transitions share the same
//! owner barrier; only the canonical subject has a commit identity.

use worth_relational::facade::durability::{
    RelationalDurabilityUnresolved, RelationalDurabilityUnresolvedSubject as Subject,
};
use worth_relational::facade::history::CommitId;

use crate::evidence_identity::WorthQueryEvidenceIdentityEncoder;
use crate::WorthQueryEvidenceTag;

/// Record the unresolved publication's subject in an evidence identity.
pub(crate) fn field_unresolved_subject(
    encoder: WorthQueryEvidenceIdentityEncoder,
    unresolved: &RelationalDurabilityUnresolved,
) -> WorthQueryEvidenceIdentityEncoder {
    match unresolved.subject() {
        Subject::InitialInputs { revision, digest } => encoder
            .field_value(
                WorthQueryEvidenceTag::new("initial_inputs_revision"),
                revision.to_string(),
            )
            .field_digest(
                WorthQueryEvidenceTag::new("initial_inputs_digest"),
                &worth_foundational::facade::CanonicalDigestId::new(digest),
            ),
        Subject::CanonicalCommit {
            position,
            commit_id,
            ..
        } => encoder
            .field_usize(
                WorthQueryEvidenceTag::new("commit_id"),
                commit_id.0 as usize,
            )
            .field_usize(WorthQueryEvidenceTag::new("position"), position.0 as usize),
        Subject::BranchTransition {
            sequence, follows, ..
        } => encoder
            .field_usize(
                WorthQueryEvidenceTag::new("branch_transition"),
                sequence.0 as usize,
            )
            .field_usize(WorthQueryEvidenceTag::new("follows"), follows.0 as usize),
    }
}

/// The unresolved commit's identity, or `None` when the unresolved
/// publication is initialization or a branch transition.
pub(crate) fn unresolved_commit_id(
    unresolved: &RelationalDurabilityUnresolved,
) -> Option<CommitId> {
    match unresolved.subject() {
        Subject::CanonicalCommit { commit_id, .. } => Some(commit_id),
        Subject::InitialInputs { .. } | Subject::BranchTransition { .. } => None,
    }
}
