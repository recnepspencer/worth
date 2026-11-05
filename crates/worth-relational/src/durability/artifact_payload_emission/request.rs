//! The same two-pass serializer lane, admitted through the original request.
use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RelationalRecoveryStop as Stop,
};
use std::io::{self, Write};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};

#[derive(Clone, Copy)]
enum Work {
    Consult,
    Charge,
}

pub(crate) fn encode_named(
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
pub(crate) fn encode_named_charged(
    prefix: &[u8],
    payload: &impl serde::Serialize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionImmutableBytes, Stop> {
    encode(prefix, request, Work::Charge, |mut writer| {
        rmp_serde::encode::write_named(&mut writer, payload)
    })
}
pub(crate) fn encode_compact(
    prefix: &[u8],
    payload: &impl serde::Serialize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionImmutableBytes, Stop> {
    encode(prefix, request, Work::Consult, |mut writer| {
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
        work: Work::Consult,
    };
    let result = rmp_serde::encode::write_named(&mut sizing, payload);
    settle(result, sizing.failure)?;
    Ok(sizing.length)
}
fn encode(
    prefix: &[u8],
    request: &ExecutionRequest<'_, '_>,
    work: Work,
    serialize: impl Fn(&mut dyn Write) -> Result<(), rmp_serde::encode::Error>,
) -> Result<ExecutionImmutableBytes, Stop> {
    let mut sizing = Sizing {
        request: *request,
        length: prefix.len(),
        failure: None,
        work,
    };
    let counted = serialize(&mut sizing);
    settle(counted, sizing.failure)?;
    let length = sizing.length;
    let mut buffer =
        ExecutionRequestArrayBuilder::allocate(*request, length).map_err(array_stop)?;
    if let Work::Charge = work {
        account_work(request, prefix.len() as u64)?;
    }
    buffer.extend_from_slice(prefix).map_err(array_stop)?;
    let mut emission = Emission {
        buffer: &mut buffer,
        failure: None,
        request: *request,
        work,
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
    work: Work,
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
        for chunk in bytes.chunks(64 * 1024) {
            let outcome = match self.work {
                Work::Consult => self
                    .request
                    .consult()
                    .map_err(|cause| Stop::Request(cause.into())),
                Work::Charge => account_work(&self.request, chunk.len() as u64),
            };
            if let Err(cause) = outcome {
                self.failure = Some(cause);
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
    request: ExecutionRequest<'r, 'a>,
    work: Work,
}
impl Write for Emission<'_, '_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if let Work::Charge = self.work
            && let Err(cause) = account_work(&self.request, bytes.len() as u64)
        {
            self.failure = Some(cause);
            return Err(stopped());
        }
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

/// Charge a bounded unit before the artifact encoder performs it. Preserve a
/// concrete inner control stop even if the SDK reports a nested scope stop.
pub(crate) fn account_work(request: &ExecutionRequest<'_, '_>, work: u64) -> Result<(), Stop> {
    let mut answer = None;
    let scope = request.run_native_with_checkpoint(|context| {
        answer = Some(crate::execution::request_stage::checkpoint(context, work));
    });
    match (scope, answer) {
        (_, Some(Err(cause))) => Err(Stop::Request(
            worth_execution::WorkCeilingDenial::Stopped(cause).into(),
        )),
        (Err(cause), _) => Err(Stop::Request(cause.into())),
        (Ok(_), Some(Ok(()))) => Ok(()),
        (Ok(_), None) => unreachable!("completed artifact work executes its callback"),
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
