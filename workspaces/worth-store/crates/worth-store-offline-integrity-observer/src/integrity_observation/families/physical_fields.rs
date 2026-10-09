use super::durable_frame::{damaged_field, DurableFrameFacts};
use crate::integrity_observation::{
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalDamageCause as Cause,
    OfflinePhysicalFormatField as Field,
};
use worth_store_physical_format::integrity_declarations::families::blob::blob_record_kind_is_declared;

#[cfg(test)]
mod tests;

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

/// The seven route-metadata bytes of a schema-3 routing entry or a classified
/// copy intent: content class, class detail, family code, tier, reserved.
pub(crate) fn route_metadata_valid(field: &[u8]) -> bool {
    if field.len() != 7 || field[5..] != [0; 2] || field[4] > 2 {
        return false;
    }
    let family = u16::from_le_bytes([field[2], field[3]]);
    match (field[0], field[1], family) {
        (0, 0, 0) => field[4] == 0,
        (1, 0, 0) | (4, 0, 0) => true,
        (2, kind, 0) => blob_record_kind_is_declared(kind),
        (3, 0, 1..) => true,
        _ => false,
    }
}

pub(crate) fn record_key(bytes: &[u8]) -> Option<([u8; 16], u64)> {
    let epoch: [u8; 16] = bytes.get(..16)?.try_into().ok()?;
    let ordinal = u64::from_le_bytes(bytes.get(16..24)?.try_into().ok()?);
    (epoch != [0; 16] && ordinal != 0).then_some((epoch, ordinal))
}
