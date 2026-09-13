use super::{
    durable_frame::{read_durable_frame, read_u16, read_u32, read_u64},
    physical_fields::{format_scope, record_key, scope, shape},
};
use crate::integrity_observation::{
    child_expectation::{ChildExpectation, ChildScope},
    OfflineIntegrityObservationCounters, OfflineIntegrityOutcome,
    OfflinePhysicalFormatField as Field,
};
use std::collections::BTreeSet;
use worth_store_physical_format::integrity_declarations::families::PAGE_FRAME_INTEGRITY_DECLARATION;

pub(crate) fn read_page_frame(
    bytes: &[u8],
    expected: &ChildExpectation,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<Vec<ChildExpectation>, OfflineIntegrityOutcome> {
    let page_bytes = read_u32(&expected.format, 2) as usize;
    let frame = read_durable_frame(
        bytes,
        page_bytes,
        3,
        PAGE_FRAME_INTEGRITY_DECLARATION,
        counters,
    )?;
    format_scope(&frame, expected.format)?;
    let ChildScope::Page { segment, page, .. } = expected.scope else {
        unreachable!()
    };
    scope(
        frame.identity == expected.generation,
        28,
        8,
        Field::FrameIdentity,
    )?;
    scope(
        read_u64(frame.payload, 0) == segment && read_u64(frame.payload, 8) == page,
        48,
        16,
        Field::IdentityField,
    )?;
    let payload = frame.payload;
    shape(payload[18..24] == [0; 6], 66, 6)?;
    let count = usize::from(read_u16(payload, 16));
    let directory_end = 24 + count * 40;
    shape(directory_end <= payload.len(), 64, 2)?;
    let mut identities = BTreeSet::new();
    let mut preceding_start = payload.len();
    for (index, slot) in payload[24..directory_end].chunks_exact(40).enumerate() {
        let offset = read_u32(slot, 24) as usize;
        let length = read_u32(slot, 28) as usize;
        let end = offset.checked_add(length);
        let key = record_key(slot);
        scope(
            key.is_some() && identities.insert(key) && read_u64(slot, 32) != 0,
            72 + index * 40,
            40,
            Field::IdentityField,
        )?;
        shape(
            offset >= directory_end && end.is_some_and(|end| end <= preceding_start),
            96 + index * 40,
            8,
        )?;
        shape(
            payload[end.unwrap()..preceding_start]
                .iter()
                .all(|value| *value == 0),
            48 + end.unwrap(),
            preceding_start.saturating_sub(end.unwrap()).max(1),
        )?;
        preceding_start = offset;
    }
    shape(
        payload[directory_end..preceding_start]
            .iter()
            .all(|value| *value == 0),
        48 + directory_end,
        preceding_start.saturating_sub(directory_end).max(1),
    )?;
    Ok(Vec::new())
}
