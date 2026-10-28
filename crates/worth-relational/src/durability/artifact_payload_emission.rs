//! Budgeted two-pass payload emission shared by Relational's framed durable
//! artifacts: count the exact encoded length under the caller's allocation
//! policy, then emit into a buffer admitted for that length. The serializer
//! is the MessagePack named-map codec.
//!
//! The canonical commit artifact and the branch transition artifact share
//! this because they share its failure topology (allocation, encoding,
//! length overflow) and its cost class (two serializer passes over one
//! payload). Each artifact maps the denial into its own encode denial.

use std::io::{self, Write};

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
    let mut sizing = PayloadSizing {
        policy,
        total: 0,
        failure: None,
    };
    let counted = rmp_serde::encode::write_named(&mut sizing, payload);
    settle(counted, sizing.failure)?;
    policy.check_live()?;
    Ok(sizing.total)
}

/// Append the payload to `buffer`, which must already hold the header and
/// have exactly the counted payload length reserved.
pub(crate) fn emit_payload(
    payload: &impl Serialize,
    buffer: &mut ExecutionByteBuffer,
) -> Result<(), ArtifactPayloadEmissionDenial> {
    let mut emission = PayloadEmission {
        buffer: &mut *buffer,
        failure: None,
    };
    let emitted = rmp_serde::encode::write_named(&mut emission, payload);
    settle(emitted, emission.failure)?;
    buffer.check_live()?;
    Ok(())
}

struct PayloadSizing<'scope, 'authority> {
    policy: ExecutionAllocationPolicy<'scope, 'authority>,
    total: usize,
    failure: Option<ArtifactPayloadEmissionDenial>,
}

impl Write for PayloadSizing<'_, '_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(stopped());
        }
        if let Err(denial) = self.policy.check_live() {
            self.failure = Some(denial.into());
            return Err(stopped());
        }
        match self
            .total
            .checked_add(input.len())
            .filter(|total| *total <= isize::MAX as usize)
        {
            Some(total) => {
                self.total = total;
                Ok(input.len())
            }
            None => {
                self.failure = Some(ArtifactPayloadEmissionDenial::FrameSizeOverflow);
                Err(stopped())
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct PayloadEmission<'a> {
    buffer: &'a mut ExecutionByteBuffer,
    failure: Option<ArtifactPayloadEmissionDenial>,
}

impl Write for PayloadEmission<'_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(stopped());
        }
        match self.buffer.extend_from_slice(input) {
            Ok(()) => Ok(input.len()),
            Err(denial) => {
                self.failure = Some(denial.into());
                Err(stopped())
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A writer's own denial outranks the serializer error it caused.
fn settle(
    result: Result<(), rmp_serde::encode::Error>,
    failure: Option<ArtifactPayloadEmissionDenial>,
) -> Result<(), ArtifactPayloadEmissionDenial> {
    if let Some(denial) = failure {
        return Err(denial);
    }
    result.map_err(|error| ArtifactPayloadEmissionDenial::PayloadEncoding {
        detail: error.to_string(),
    })
}

fn stopped() -> io::Error {
    io::Error::other("artifact payload writer stopped")
}
