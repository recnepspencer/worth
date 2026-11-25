use std::sync::Arc;

use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;

/// Owner-issued admission for binding one canonical envelope to one runtime
/// patch-stream position. Its fields and constructors stay inside the
/// canonical publication owner; history data can only consume the seal.
pub struct CanonicalPositionAdmission {
    position: PatchStreamPosition,
    canonical: Arc<CanonicalCommitEnvelope>,
}

pub use crate::publication::patch::data::position_denial::RelationalCanonicalPositionAdmissionDenial;

impl CanonicalPositionAdmission {
    pub fn equivalent(
        original: &PositionedCanonicalCommit,
        canonical: Arc<CanonicalCommitEnvelope>,
    ) -> Result<Self, &'static str> {
        if !crate::history::commit::payload_commitment::matches_positioned_payload(
            original, &canonical,
        )
        .map_err(|_| "recovered position payload encoding failed")?
        {
            return Err("recovered position body differs from its admitted canonical payload");
        }
        Ok(Self {
            position: original.position(),
            canonical,
        })
    }
    pub(super) fn retained(
        retention: &crate::runtime::state::subsystems::history::canonical_publication_routes::retained_position::RetainedCanonicalPosition,
    ) -> Option<Self> {
        let (position, canonical) = retention.upgrade_parts()?;
        Some(Self {
            position,
            canonical,
        })
    }
    pub fn performed(
        position: PatchStreamPosition,
        canonical: Arc<CanonicalCommitEnvelope>,
    ) -> Self {
        Self {
            position,
            canonical,
        }
    }

    pub fn into_parts(self) -> (PatchStreamPosition, Arc<CanonicalCommitEnvelope>) {
        (self.position, self.canonical)
    }
}

pub fn readmit_positioned_canonical_commit(
    position: PatchStreamPosition,
    canonical: Arc<CanonicalCommitEnvelope>,
) -> Result<PositionedCanonicalCommit, RelationalCanonicalPositionAdmissionDenial> {
    if position.0 == 0 {
        return Err(RelationalCanonicalPositionAdmissionDenial::PositionZero);
    }
    if canonical.branch_context != canonical.commit.branch_id {
        return Err(RelationalCanonicalPositionAdmissionDenial::BranchContextMismatch);
    }
    crate::transactions::data::outcomes::commit_semantic_result::readmission::validate_envelope(
        canonical.semantic_result(),
        &canonical,
    )
    .map_err(RelationalCanonicalPositionAdmissionDenial::SemanticResult)?;
    crate::history::commit::RelationalCommitCatalog::validate_detached_envelope(&canonical)
        .map_err(RelationalCanonicalPositionAdmissionDenial::CommitArtifact)?;
    Ok(PositionedCanonicalCommit::admit(
        CanonicalPositionAdmission {
            position,
            canonical,
        },
    ))
}
