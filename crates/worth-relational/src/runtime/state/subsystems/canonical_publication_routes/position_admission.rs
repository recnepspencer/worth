use std::sync::Arc;

use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::history::RelationalCommitArtifactDenial;
use crate::publication::patch::data::PatchStreamPosition;

/// Owner-issued admission for binding one canonical envelope to one runtime
/// patch-stream position. Its fields and constructors stay inside the
/// canonical publication owner; history data can only consume the seal.
pub(crate) struct CanonicalPositionAdmission {
    position: PatchStreamPosition,
    canonical: Arc<CanonicalCommitEnvelope>,
}

/// Why a decoded canonical envelope could not be readmitted at a stream
/// position. Each variant names the violated admission contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelationalCanonicalPositionAdmissionDenial {
    /// Position zero precedes every published commit and is never assigned.
    PositionZero,
    /// The envelope's branch context differs from its commit provenance.
    BranchContextMismatch,
    /// The envelope fails its own commit-artifact invariants.
    CommitArtifact(RelationalCommitArtifactDenial),
}

impl CanonicalPositionAdmission {
    pub(super) fn performed(
        position: PatchStreamPosition,
        canonical: Arc<CanonicalCommitEnvelope>,
    ) -> Self {
        Self {
            position,
            canonical,
        }
    }

    pub(crate) fn into_parts(self) -> (PatchStreamPosition, Arc<CanonicalCommitEnvelope>) {
        (self.position, self.canonical)
    }
}

pub(crate) fn readmit_positioned_canonical_commit(
    position: PatchStreamPosition,
    canonical: Arc<CanonicalCommitEnvelope>,
) -> Result<PositionedCanonicalCommit, RelationalCanonicalPositionAdmissionDenial> {
    if position.0 == 0 {
        return Err(RelationalCanonicalPositionAdmissionDenial::PositionZero);
    }
    if canonical.branch_context != canonical.commit.branch_id {
        return Err(RelationalCanonicalPositionAdmissionDenial::BranchContextMismatch);
    }
    crate::history::RelationalCommitCatalog::validate_detached_envelope(&canonical)
        .map_err(RelationalCanonicalPositionAdmissionDenial::CommitArtifact)?;
    Ok(PositionedCanonicalCommit::admit(
        CanonicalPositionAdmission {
            position,
            canonical,
        },
    ))
}

impl std::fmt::Display for RelationalCanonicalPositionAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PositionZero => {
                formatter.write_str("canonical patch-stream position zero is not admissible")
            }
            Self::BranchContextMismatch => formatter
                .write_str("canonical envelope branch context does not match commit provenance"),
            Self::CommitArtifact(denial) => {
                write!(formatter, "canonical envelope admission failed: {denial:?}")
            }
        }
    }
}

impl std::error::Error for RelationalCanonicalPositionAdmissionDenial {}
