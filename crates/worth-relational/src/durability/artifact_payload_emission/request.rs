//! The same two-pass serializer lane, admitted through the original request.
use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RelationalRecoveryStop as Stop,
};
use std::io::{self, Write};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};

pub(crate) fn encode_named(
    prefix: &[u8],
    payload: &impl serde::Serialize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionImmutableBytes, Stop> {
    encode(prefix, request, |mut writer| {
        rmp_serde::encode::write_named(&mut writer, payload)
    })
}
pub(crate) fn encode_compact(
    prefix: &[u8],
    payload: &impl serde::Serialize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionImmutableBytes, Stop> {
    encode(prefix, request, |mut writer| {
        rmp_serde::encode::write(&mut writer, payload)
    })
}
pub(crate) fn named_size(
    payload: &impl serde::Serialize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<usize, Stop> {
    let mut sizing = Sizing {
        request: *request,
        length: 0,
        failure: None,
    };
    let result = rmp_serde::encode::write_named(&mut sizing, payload);
    settle(result, sizing.failure)?;
    Ok(sizing.length)
}
fn encode(
    prefix: &[u8],
    request: &ExecutionRequest<'_, '_>,
    serialize: impl Fn(&mut dyn Write) -> Result<(), rmp_serde::encode::Error>,
) -> Result<ExecutionImmutableBytes, Stop> {
    let mut sizing = Sizing {
        request: *request,
        length: prefix.len(),
        failure: None,
    };
    let counted = serialize(&mut sizing);
    settle(counted, sizing.failure)?;
    let length = sizing.length;
    let mut buffer =
        ExecutionRequestArrayBuilder::allocate(*request, length).map_err(array_stop)?;
    buffer.extend_from_slice(prefix).map_err(array_stop)?;
    let mut emission = Emission {
        buffer: &mut buffer,
        failure: None,
    };
    let emitted = serialize(&mut emission);
    settle(emitted, emission.failure)?;
    if buffer.len() != length {
        return Err(corrupt(
            "counted and emitted checkpoint payload lengths differ",
        ));
    }
    buffer.seal_immutable_bytes().map_err(array_stop)
}
struct Sizing<'r, 'a> {
    request: ExecutionRequest<'r, 'a>,
    length: usize,
    failure: Option<Stop>,
}
impl Write for Sizing<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let Some(next) = self
            .length
            .checked_add(bytes.len())
            .filter(|length| *length <= isize::MAX as usize)
        else {
            self.failure = Some(Stop::Request(
                crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
            ));
            return Err(stopped());
        };
        for _ in bytes.chunks(64 * 1024) {
            if let Err(cause) = self.request.consult() {
                self.failure = Some(Stop::Request(cause.into()));
                return Err(stopped());
            }
        }
        self.length = next;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct Emission<'b, 'r, 'a> {
    buffer: &'b mut ExecutionRequestArrayBuilder<'r, 'a, u8>,
    failure: Option<Stop>,
}
impl Write for Emission<'_, '_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self.buffer.extend_from_slice(bytes) {
            Ok(()) => Ok(bytes.len()),
            Err(cause) => {
                self.failure = Some(array_stop(cause));
                Err(stopped())
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn settle(result: Result<(), rmp_serde::encode::Error>, failure: Option<Stop>) -> Result<(), Stop> {
    if let Some(cause) = failure {
        return Err(cause);
    }
    result.map_err(|_| corrupt("checkpoint payload serializer refused its producer data"))
}
fn array_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> Stop {
    match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => Stop::Request(cause.into()),
        worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => Stop::Allocation(cause),
    }
}
fn corrupt(detail: &str) -> Stop {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail).into()
}
fn stopped() -> io::Error {
    io::ErrorKind::Other.into()
}
