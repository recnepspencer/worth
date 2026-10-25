use super::RelationalBranchTransitionArtifactFrameRefusal;
use crate::branch::RelationalBranchTransitionSequence;
use crate::execution::RelationalExecutionDenialCause;
use crate::publication::patch::data::PatchStreamPosition;

/// Why artifact bytes were not readmitted as a positioned branch transition.
/// Variants follow readmission order; each refuses before any later phase
/// runs and none falls back to another format.
#[derive(Debug)]
#[non_exhaustive]
pub enum RelationalBranchTransitionArtifactRefusal {
    /// The execution request was already stopped before readmission began.
    Request(RelationalExecutionDenialCause),
    /// The frame, version, length or payload digest is not intact.
    Frame(RelationalBranchTransitionArtifactFrameRefusal),
    /// The digested payload is not a current transition payload.
    PayloadDecode { detail: String },
    /// The header sequence differs from the sequence inside the payload.
    SequenceDisagreement {
        header: RelationalBranchTransitionSequence,
        payload: RelationalBranchTransitionSequence,
    },
    /// The header's followed position differs from the payload's.
    FollowsDisagreement {
        header: PatchStreamPosition,
        payload: PatchStreamPosition,
    },
    /// Sequence zero precedes every transition and is never assigned.
    SequenceZero,
    /// A fork's source head is bound to a live runtime rather than the
    /// canonical runtime frame.
    SourceHeadOutsideCanonicalFrame { runtime_instance_id: u64 },
    /// The payload decodes to a transition but is not that transition's
    /// canonical encoding, so its digest could not identify it.
    NonCanonicalPayload,
}

impl From<RelationalBranchTransitionArtifactFrameRefusal>
    for RelationalBranchTransitionArtifactRefusal
{
    fn from(refusal: RelationalBranchTransitionArtifactFrameRefusal) -> Self {
        Self::Frame(refusal)
    }
}

impl std::fmt::Display for RelationalBranchTransitionArtifactRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(cause) => write!(
                formatter,
                "branch transition artifact readmission stopped: {cause:?}"
            ),
            Self::Frame(refusal) => refusal.fmt(formatter),
            Self::PayloadDecode { detail } => write!(
                formatter,
                "branch transition artifact payload is undecodable: {detail}"
            ),
            Self::SequenceDisagreement { header, payload } => write!(
                formatter,
                "branch transition artifact header sequence {} disagrees with payload sequence {}",
                header.0, payload.0
            ),
            Self::FollowsDisagreement { header, payload } => write!(
                formatter,
                "branch transition artifact header follows position {} but its payload follows {}",
                header.0, payload.0
            ),
            Self::SequenceZero => {
                formatter.write_str("branch transition sequence zero is not admissible")
            }
            Self::SourceHeadOutsideCanonicalFrame {
                runtime_instance_id,
            } => write!(
                formatter,
                "fork source head is bound to runtime {runtime_instance_id}, not the canonical frame"
            ),
            Self::NonCanonicalPayload => formatter.write_str(
                "branch transition artifact payload is not the canonical transition encoding",
            ),
        }
    }
}

impl std::error::Error for RelationalBranchTransitionArtifactRefusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Frame(refusal) => Some(refusal),
            _ => None,
        }
    }
}
