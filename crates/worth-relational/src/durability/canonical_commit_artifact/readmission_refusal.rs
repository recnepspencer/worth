use super::RelationalCanonicalCommitArtifactFrameRefusal;
use crate::execution::RelationalExecutionDenialCause;
use crate::history::data::CommitId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalCanonicalPositionAdmissionDenial;
use crate::symbols::data::RelationalFieldTextAdmissionDenial;

/// Why artifact bytes were not readmitted as a canonical commit. Variants
/// follow readmission order; each refuses before any later phase runs and
/// none falls back to another format or a partial commit.
#[derive(Debug)]
#[non_exhaustive]
pub enum RelationalCanonicalCommitArtifactRefusal {
    /// The execution request was already stopped before readmission began.
    Request(RelationalExecutionDenialCause),
    /// The frame, version, length or payload digest is not intact.
    Frame(RelationalCanonicalCommitArtifactFrameRefusal),
    /// The digested payload is not a current artifact payload.
    PayloadDecode { detail: String },
    /// The header position differs from the position inside the payload.
    PositionDisagreement {
        header: PatchStreamPosition,
        payload: PatchStreamPosition,
    },
    /// The header commit id differs from the envelope's commit id.
    CommitIdDisagreement {
        header: CommitId,
        envelope: CommitId,
    },
    /// The envelope's managed field text could not be readmitted.
    FieldText(RelationalFieldTextAdmissionDenial),
    /// The envelope failed canonical admission at its position.
    PositionAdmission(RelationalCanonicalPositionAdmissionDenial),
    /// The payload decodes to an admissible commit but is not that commit's
    /// canonical encoding, so its digest could not identify the commit.
    NonCanonicalPayload,
}

impl From<RelationalCanonicalCommitArtifactFrameRefusal>
    for RelationalCanonicalCommitArtifactRefusal
{
    fn from(refusal: RelationalCanonicalCommitArtifactFrameRefusal) -> Self {
        Self::Frame(refusal)
    }
}

impl std::fmt::Display for RelationalCanonicalCommitArtifactRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(cause) => write!(
                formatter,
                "canonical commit artifact readmission stopped: {cause:?}"
            ),
            Self::Frame(refusal) => refusal.fmt(formatter),
            Self::PayloadDecode { detail } => write!(
                formatter,
                "canonical commit artifact payload is undecodable: {detail}"
            ),
            Self::PositionDisagreement { header, payload } => write!(
                formatter,
                "canonical commit artifact header position {} disagrees with payload position {}",
                header.0, payload.0
            ),
            Self::CommitIdDisagreement { header, envelope } => write!(
                formatter,
                "canonical commit artifact header commit {} disagrees with envelope commit {}",
                header.0, envelope.0
            ),
            Self::FieldText(denial) => write!(
                formatter,
                "canonical commit artifact field text refused: {denial:?}"
            ),
            Self::PositionAdmission(denial) => denial.fmt(formatter),
            Self::NonCanonicalPayload => formatter.write_str(
                "canonical commit artifact payload is not the canonical commit encoding",
            ),
        }
    }
}

impl std::error::Error for RelationalCanonicalCommitArtifactRefusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Frame(refusal) => Some(refusal),
            Self::PositionAdmission(denial) => Some(denial),
            _ => None,
        }
    }
}
