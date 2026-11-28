//! The same two-pass serializer lane, admitted through the original request.
use crate::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use crate::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
use std::io::{self, Write};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};

#[derive(Clone, Copy)]
enum Work {
    Consult,
    Charge,
}

pub fn encode_named(
    prefix: &[u8],
    payload: &impl serde::Serialize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionImmutableBytes, Stop> {
    encode(prefix, request, Work::Consult, |mut writer| {
        rmp_serde::encode::write_named(&mut writer, payload)
    })
}

/// Charge both actual serializer traversals and emitted copies before writing.
/// Existing callers retain their declared consult-only encoding contract.
pub fn encode_named_charged(
    prefix: &[u8],
    payload: &impl serde::Serialize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionImmutableBytes, Stop> {
    encode(prefix, request, Work::Charge, |mut writer| {
        rmp_serde::encode::write_named(&mut writer, payload)
    })
}
pub fn encode_compact(
    prefix: &[u8],
    payload: &impl serde::Serialize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionImmutableBytes, Stop> {
    encode(prefix, request, Work::Consult, |mut writer| {
        rmp_serde::encode::write(&mut writer, payload)
    })
}
pub fn named_size(
    payload: &impl serde::Serialize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<usize, Stop> {
    let mut length = 0;
    named_to(
        payload,
        |bytes| size(&mut length, bytes, request, Work::Consult),
        encode_stop,
    )?;
    Ok(length)
}

/// Run a named MessagePack pass against typed caller-owned storage or metering.
pub fn named_to<E>(
    value: &impl serde::Serialize,
    write: impl FnMut(&[u8]) -> Result<(), E>,
    refused: impl FnOnce(rmp_serde::encode::Error) -> E,
) -> Result<(), E> {
    serialize_to(
        |writer| rmp_serde::encode::write_named(&mut &mut *writer, value),
        write,
        refused,
    )
}

fn serialize_to<E>(
    serialize: impl FnOnce(&mut dyn Write) -> Result<(), rmp_serde::encode::Error>,
    write: impl FnMut(&[u8]) -> Result<(), E>,
    refused: impl FnOnce(rmp_serde::encode::Error) -> E,
) -> Result<(), E> {
    let mut writer = PayloadWriter {
        write,
        failure: None,
    };
    let encoded = serialize(&mut writer);
    match writer.failure {
        Some(cause) => Err(cause),
        None => encoded.map_err(refused),
    }
}

struct PayloadWriter<F, E> {
    write: F,
    failure: Option<E>,
}
impl<F: FnMut(&[u8]) -> Result<(), E>, E> Write for PayloadWriter<F, E> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::ErrorKind::Other.into());
        }
        (self.write)(bytes).map(|()| bytes.len()).map_err(|cause| {
            self.failure = Some(cause);
            io::ErrorKind::Other.into()
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Encoded payload lengths must fit the allocation and slice address space.
pub fn extend_length(length: usize, bytes: &[u8]) -> Option<usize> {
    length
        .checked_add(bytes.len())
        .filter(|length| *length <= isize::MAX as usize)
}

fn size(
    length: &mut usize,
    bytes: &[u8],
    request: &ExecutionRequest<'_, '_>,
    work: Work,
) -> Result<(), Stop> {
    let next = extend_length(*length, bytes).ok_or(Stop::Request(
        crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
    ))?;
    for chunk in bytes.chunks(64 * 1024) {
        match work {
            Work::Consult => request
                .consult()
                .map_err(|cause| Stop::Request(cause.into()))?,
            Work::Charge => account_work(request, chunk.len() as u64)?,
        }
    }
    *length = next;
    Ok(())
}

fn encode(
    prefix: &[u8],
    request: &ExecutionRequest<'_, '_>,
    work: Work,
    serialize: impl Fn(&mut dyn Write) -> Result<(), rmp_serde::encode::Error>,
) -> Result<ExecutionImmutableBytes, Stop> {
    let mut length = prefix.len();
    serialize_to(
        &serialize,
        |bytes| size(&mut length, bytes, request, work),
        encode_stop,
    )?;
    let mut buffer =
        ExecutionRequestArrayBuilder::allocate(*request, length).map_err(array_stop)?;
    if let Work::Charge = work {
        account_work(request, prefix.len() as u64)?;
    }
    buffer.extend_from_slice(prefix).map_err(array_stop)?;
    serialize_to(
        serialize,
        |bytes| {
            if let Work::Charge = work {
                account_work(request, bytes.len() as u64)?;
            }
            buffer.extend_from_slice(bytes).map_err(array_stop)
        },
        encode_stop,
    )?;
    if buffer.len() != length {
        return Err(corrupt(
            "counted and emitted checkpoint payload lengths differ",
        ));
    }
    buffer.seal_immutable_bytes().map_err(array_stop)
}

/// Charge a bounded unit before the artifact encoder performs it. Preserve a
/// concrete inner control stop even if the SDK reports a nested scope stop.
pub use crate::durability::data::recovery_stop::account_work;

fn encode_stop(_: rmp_serde::encode::Error) -> Stop {
    corrupt("checkpoint payload serializer refused its producer data")
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
