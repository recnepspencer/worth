use worth_execution::ExecutionAllocationDenial;

/// Why a positioned branch transition could not be framed as an artifact.
/// Encoding is all-or-nothing: a denial leaves no partial artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelationalBranchTransitionArtifactEncodeDenial {
    /// The caller's allocation policy refused, was cancelled or expired.
    Allocation(ExecutionAllocationDenial),
    /// The transition failed to serialize.
    PayloadEncoding { detail: String },
    /// Header plus payload length is not representable.
    FrameSizeOverflow,
    /// The counting pass and the emitting pass disagreed on payload length.
    CountEmissionMismatch { counted: usize, emitted: usize },
}

impl From<ExecutionAllocationDenial> for RelationalBranchTransitionArtifactEncodeDenial {
    fn from(denial: ExecutionAllocationDenial) -> Self {
        Self::Allocation(denial)
    }
}

impl From<crate::durability::artifact_payload_emission::ArtifactPayloadEmissionDenial>
    for RelationalBranchTransitionArtifactEncodeDenial
{
    fn from(
        denial: crate::durability::artifact_payload_emission::ArtifactPayloadEmissionDenial,
    ) -> Self {
        use crate::durability::artifact_payload_emission::ArtifactPayloadEmissionDenial as Emission;
        match denial {
            Emission::Allocation(denial) => Self::Allocation(denial),
            Emission::PayloadEncoding { detail } => Self::PayloadEncoding { detail },
            Emission::FrameSizeOverflow => Self::FrameSizeOverflow,
        }
    }
}

impl std::fmt::Display for RelationalBranchTransitionArtifactEncodeDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allocation(denial) => {
                write!(formatter, "branch transition artifact allocation: {denial}")
            }
            Self::PayloadEncoding { detail } => {
                write!(
                    formatter,
                    "branch transition artifact payload encoding: {detail}"
                )
            }
            Self::FrameSizeOverflow => {
                formatter.write_str("branch transition artifact size is not representable")
            }
            Self::CountEmissionMismatch { counted, emitted } => write!(
                formatter,
                "branch transition artifact counted {counted} payload bytes but emitted {emitted}"
            ),
        }
    }
}

impl std::error::Error for RelationalBranchTransitionArtifactEncodeDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(denial) => Some(denial),
            _ => None,
        }
    }
}
