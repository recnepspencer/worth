//! World tags actual component owners and transports Signal's opaque envelope.
use super::fields::{evidence, number, Emit, Read};
use crate::durability::artifact::{WorldOccurrenceOwnerEvidence as Owners, WorldSignalEvidence};
use crate::durability::{WorldDurableFrameDenial as Frame, WorldDurableMediumDenial as Medium};
use worth_execution::ExecutionRequest;

pub(in crate::durability) fn write(out: &mut Emit<'_>, owners: &Owners) -> Result<(), Frame> {
    match owners {
        Owners::RelationalOnly(relational) => {
            out(&[1])?;
            evidence(out, relational)
        }
        Owners::SignalOnly(signal) => {
            out(&[2])?;
            signal_bytes(out, signal)
        }
        Owners::RelationalAndSignal { relational, signal } => {
            out(&[3])?;
            evidence(out, relational)?;
            signal_bytes(out, signal)
        }
    }
}
pub(super) fn signal_bytes(out: &mut Emit<'_>, signal: &WorldSignalEvidence) -> Result<(), Frame> {
    const MAXIMUM: usize =
        worth_signal::facade::durability::SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES;
    let unused = MAXIMUM
        .checked_sub(signal.as_bytes().len())
        .ok_or(Frame::SizeLimit)?;
    number(out, signal.as_bytes().len() as u64)?;
    out(signal.as_bytes())?;
    // One fixed transport slot is prepaid before either owner has an effect.
    // Its zero padding is World framing, outside Signal's checksummed envelope.
    out(&[0u8; MAXIMUM][..unused])
}
pub(in crate::durability) fn read(
    input: &mut Read<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Owners, Medium> {
    Ok(match input.fixed::<1>()? {
        [1] => Owners::RelationalOnly(input.evidence()?),
        [2] => Owners::SignalOnly(read_signal(input, request)?),
        [3] => Owners::RelationalAndSignal {
            relational: input.evidence()?,
            signal: read_signal(input, request)?,
        },
        _ => return Err(Frame::Malformed.into()),
    })
}
pub(super) fn read_signal(
    input: &mut Read<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<WorldSignalEvidence, Medium> {
    let length = usize::try_from(input.number()?).map_err(|_| Frame::SizeLimit)?;
    if length > worth_signal::facade::durability::SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES {
        return Err(Frame::SizeLimit.into());
    }
    let slot = input
        .take(worth_signal::facade::durability::SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES)?;
    WorldSignalEvidence::from_bytes(&slot[..length], request)
}
