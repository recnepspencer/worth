use worth_execution::ExecutionAllocationDenial;

/// Why a positioned canonical commit could not be framed as an artifact.
/// Encoding is all-or-nothing: a denial leaves no partial artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelationalCanonicalCommitArtifactEncodeDenial {
    Readmission(crate::durability::data::recovery_stop::RelationalRecoveryStop),
    Frame(crate::durability::canonical_commit_artifact::frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal),
    Request(crate::execution::denial_cause::RelationalExecutionDenialCause),
    StorageUnavailable(crate::storage::data::storage_unavailable::RelationalStorageUnavailable),
    /// The caller's allocation policy refused, was cancelled or expired.
    Allocation(ExecutionAllocationDenial),
    /// The envelope projection failed to serialize.
    PayloadEncoding {
        detail: String,
    },
    /// Header plus payload length is not representable.
    FrameSizeOverflow,
    SizeLimit {
        observed: usize,
        maximum: usize,
    },
    /// The counting pass and the emitting pass disagreed on payload length.
    CountEmissionMismatch {
        counted: usize,
        emitted: usize,
    },
}

impl From<ExecutionAllocationDenial> for RelationalCanonicalCommitArtifactEncodeDenial {
    fn from(denial: ExecutionAllocationDenial) -> Self {
        Self::Allocation(denial)
    }
}

impl std::fmt::Display for RelationalCanonicalCommitArtifactEncodeDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Readmission(cause) => write!(formatter, "selected canonical source: {cause}"),
            Self::Frame(cause) => write!(formatter, "canonical artifact source frame: {cause:?}"),
            Self::Request(cause) => {
                write!(formatter, "canonical artifact original request: {cause:?}")
            }
            Self::StorageUnavailable(failure) => write!(
                formatter,
                "canonical artifact storage unavailable: {failure:?}"
            ),
            Self::Allocation(denial) => {
                write!(formatter, "canonical commit artifact allocation: {denial}")
            }
            Self::PayloadEncoding { detail } => {
                write!(
                    formatter,
                    "canonical commit artifact payload encoding: {detail}"
                )
            }
            Self::SizeLimit { observed, maximum } => write!(
                formatter,
                "canonical artifact size {observed} exceeds {maximum}"
            ),
            Self::FrameSizeOverflow => {
                formatter.write_str("canonical commit artifact size is not representable")
            }
            Self::CountEmissionMismatch { counted, emitted } => write!(
                formatter,
                "canonical commit artifact counted {counted} payload bytes but emitted {emitted}"
            ),
        }
    }
}

impl std::error::Error for RelationalCanonicalCommitArtifactEncodeDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(denial) => Some(denial),
            _ => None,
        }
    }
}

impl From<worth_execution::ExecutionRequestArrayDenial>
    for RelationalCanonicalCommitArtifactEncodeDenial
{
    fn from(cause: worth_execution::ExecutionRequestArrayDenial) -> Self {
        match cause {
            worth_execution::ExecutionRequestArrayDenial::Request(cause) => {
                Self::Request(cause.into())
            }
            worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => {
                Self::Allocation(cause)
            }
        }
    }
}
