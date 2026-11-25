use crate::history::commit::artifact_denial::RelationalCommitArtifactDenial;
use crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResultRefusal;

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
    SemanticResult(RelationalCommitSemanticResultRefusal),
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
            Self::SemanticResult(denial) => write!(
                formatter,
                "canonical semantic result admission failed: {denial:?}"
            ),
        }
    }
}

impl std::error::Error for RelationalCanonicalPositionAdmissionDenial {}
