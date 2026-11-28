//! Budgeted two-pass payload emission shared by Relational's framed durable
//! artifacts: count the exact encoded length under the caller's allocation
//! policy, then emit into a buffer admitted for that length. The serializer
//! is the MessagePack named-map codec.
//!
//! The canonical commit artifact and the branch transition artifact share
//! this because they share its failure topology (allocation, encoding,
//! length overflow) and its cost class (two serializer passes over one
//! payload). Each artifact maps the denial into its own encode denial.

pub use contracts::durability::artifact_payload_emission::request;
use worth_relational_contracts::internal_port as contracts;

use serde::Serialize;
use worth_execution::{ExecutionAllocationDenial, ExecutionAllocationPolicy, ExecutionByteBuffer};

/// Why a payload could not be counted or emitted. Emission is all-or-nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArtifactPayloadEmissionDenial {
    /// The caller's allocation policy refused, was cancelled or expired.
    Allocation(ExecutionAllocationDenial),
    /// The payload failed to serialize.
    PayloadEncoding { detail: String },
    /// The payload length is not representable.
    FrameSizeOverflow,
}

impl From<ExecutionAllocationDenial> for ArtifactPayloadEmissionDenial {
    fn from(denial: ExecutionAllocationDenial) -> Self {
        Self::Allocation(denial)
    }
}

/// Encoded payload length. Checks liveness on every write so a cancelled or
/// expired request stops before the allocation is quoted.
pub(crate) fn count_payload(
    payload: &impl Serialize,
    policy: ExecutionAllocationPolicy<'_, '_>,
) -> Result<usize, ArtifactPayloadEmissionDenial> {
    let mut length = 0;
    request::named_to(
        payload,
        |bytes| {
            policy.check_live()?;
            length = request::extend_length(length, bytes)
                .ok_or(ArtifactPayloadEmissionDenial::FrameSizeOverflow)?;
            Ok(())
        },
        encoding_failure,
    )?;
    policy.check_live()?;
    Ok(length)
}

/// Append the payload to the admitted buffer after the framed header.
pub(crate) fn emit_payload(
    payload: &impl Serialize,
    buffer: &mut ExecutionByteBuffer,
) -> Result<(), ArtifactPayloadEmissionDenial> {
    request::named_to(
        payload,
        |bytes| buffer.extend_from_slice(bytes).map_err(Into::into),
        encoding_failure,
    )?;
    buffer.check_live()?;
    Ok(())
}

fn encoding_failure(error: rmp_serde::encode::Error) -> ArtifactPayloadEmissionDenial {
    ArtifactPayloadEmissionDenial::PayloadEncoding {
        detail: error.to_string(),
    }
}

impl From<crate::durability::artifact_payload_emission::ArtifactPayloadEmissionDenial>
    for contracts::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial
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

impl From<crate::durability::artifact_payload_emission::ArtifactPayloadEmissionDenial>
    for contracts::durability::branch_transition_artifact::encode_denial::RelationalBranchTransitionArtifactEncodeDenial
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
