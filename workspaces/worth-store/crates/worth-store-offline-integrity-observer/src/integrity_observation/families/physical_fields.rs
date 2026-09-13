use super::durable_frame::{damaged_field, DurableFrameFacts};
use crate::integrity_observation::{
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalDamageCause as Cause,
    OfflinePhysicalFormatField as Field,
};

pub(crate) fn scope(
    condition: bool,
    offset: usize,
    length: usize,
    field: Field,
) -> Result<(), Outcome> {
    if condition {
        Ok(())
    } else {
        Err(damaged_field(
            Cause::ScopeMismatch,
            offset as u64,
            length as u64,
            field,
        ))
    }
}

pub(crate) fn shape(condition: bool, offset: usize, length: usize) -> Result<(), Outcome> {
    if condition {
        Ok(())
    } else {
        Err(damaged_field(
            Cause::Framing,
            offset as u64,
            length as u64,
            Field::PayloadLength,
        ))
    }
}

pub(crate) fn format_scope(
    frame: &DurableFrameFacts<'_>,
    expected: [u8; 10],
) -> Result<(), Outcome> {
    scope(frame.format == expected, 10, 10, Field::EmbeddedFormat)
}

pub(crate) fn record_key(bytes: &[u8]) -> Option<([u8; 16], u64)> {
    let epoch: [u8; 16] = bytes.get(..16)?.try_into().ok()?;
    let ordinal = u64::from_le_bytes(bytes.get(16..24)?.try_into().ok()?);
    (epoch != [0; 16] && ordinal != 0).then_some((epoch, ordinal))
}
