//! Budgeted two-pass payload emission: count the exact encoded length under
//! the caller's allocation policy, then emit into a buffer admitted for that
//! length. The serializer is the envelope's MessagePack named-map codec.

use std::io::{self, Write};

use serde::Serialize;
use worth_execution::{ExecutionAllocationPolicy, ExecutionByteBuffer};

use super::RelationalCanonicalCommitArtifactEncodeDenial as Denial;

/// Encoded payload length. Checks liveness on every write so a cancelled or
/// expired request stops before the allocation is quoted.
pub(super) fn count_payload(
    payload: &impl Serialize,
    policy: ExecutionAllocationPolicy<'_, '_>,
) -> Result<usize, Denial> {
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
pub(super) fn emit_payload(
    payload: &impl Serialize,
    buffer: &mut ExecutionByteBuffer,
) -> Result<(), Denial> {
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
    failure: Option<Denial>,
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
                self.failure = Some(Denial::FrameSizeOverflow);
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
    failure: Option<Denial>,
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
    failure: Option<Denial>,
) -> Result<(), Denial> {
    if let Some(denial) = failure {
        return Err(denial);
    }
    result.map_err(|error| Denial::PayloadEncoding {
        detail: error.to_string(),
    })
}

fn stopped() -> io::Error {
    io::Error::other("canonical commit artifact payload writer stopped")
}
