//! Optional trailing archive extension for whole-result live subscriptions.
//! Queries without `live_whole_result` emit zero trailing bytes, preserving
//! exact prior wire bytes.
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use worth_query_declaration::facade::application_query::WorthQueryPortableApplicationQueryParts;

const MARKER: &[u8; 4] = b"WQWL";
const VERSION: u16 = 1;
const WHOLE_RESULT_MODE: u8 = 1;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    parts: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), Denial> {
    if !parts.is_live_whole_result() {
        return Ok(());
    }
    if !parts.lanes().live_enabled()
        || parts.live_cause().is_some()
        || parts.orbit().is_some()
        || parts.component_selection().is_some()
        || parts.managed_index().is_some()
        || parts.continuation().is_some()
    {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    output.raw_bytes(MARKER)?;
    output.u16(VERSION)?;
    output.u8(WHOLE_RESULT_MODE)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    live_enabled: bool,
    has_live_cause: bool,
    has_unsupported_shape: bool,
) -> Result<bool, Denial> {
    if !input.remaining_starts_with(MARKER) {
        return Ok(false);
    }
    input.take(MARKER.len())?;
    if input.u16()? != VERSION {
        return Err(Denial::new(Kind::UnsupportedRecordVersion));
    }
    match input.u8()? {
        WHOLE_RESULT_MODE => {}
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
    if !live_enabled || has_live_cause || has_unsupported_shape {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    Ok(true)
}
