use super::super::{
    OfflineIntegrityObservationCounters, OfflineIntegrityOutcome, OfflinePhysicalDamageCause,
    OfflinePhysicalFormatField,
};
use super::durable_frame::{damaged_field, read_durable_frame, read_u16, read_u32, read_u64};
use worth_store_physical_format::integrity_declarations::families::root::ROOT_MANIFEST_INTEGRITY_DECLARATION;

pub(crate) const ROOT_MANIFEST_BYTES: usize = 384;
pub(crate) const DIRECTORY_BOUND_ROOT_MANIFEST_BYTES: usize = 544;
pub(crate) const QUARANTINE_BOUND_ROOT_MANIFEST_BYTES: usize = 576;
pub(crate) const TIER_ANCHORED_ROOT_MANIFEST_BYTES: usize = 608;
const ROOT_MANIFEST_KIND: u8 = 2;

/// The canonical frame scope comes from the envelope schema, even when the
/// acquired file is truncated. Unknown or unavailable schema keeps the legacy
/// root scope rather than claiming the observed byte count is authoritative.
pub(crate) fn expected_root_manifest_bytes(bytes: &[u8]) -> usize {
    match bytes.get(9) {
        Some(4 | 5) => DIRECTORY_BOUND_ROOT_MANIFEST_BYTES,
        Some(6 | 7) => QUARANTINE_BOUND_ROOT_MANIFEST_BYTES,
        Some(9) => TIER_ANCHORED_ROOT_MANIFEST_BYTES,
        _ => ROOT_MANIFEST_BYTES,
    }
}

#[cfg(test)]
mod canonical_scope_tests {
    use super::*;

    #[test]
    fn root_scope_uses_schema_not_truncated_acquisition_length() {
        for (schema, expected) in [
            (2, ROOT_MANIFEST_BYTES),
            (4, DIRECTORY_BOUND_ROOT_MANIFEST_BYTES),
            (7, QUARANTINE_BOUND_ROOT_MANIFEST_BYTES),
            (8, ROOT_MANIFEST_BYTES),
            (9, TIER_ANCHORED_ROOT_MANIFEST_BYTES),
        ] {
            let mut truncated = [0_u8; 10];
            truncated[9] = schema;
            assert_eq!(expected_root_manifest_bytes(&truncated), expected);
        }
        assert_eq!(expected_root_manifest_bytes(&[]), ROOT_MANIFEST_BYTES);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OfflineRootManifestFacts {
    pub(crate) generation: u64,
    pub(crate) format: [u8; 10],
    pub(crate) payload: [u8; 560],
    pub(crate) encoded_bytes: usize,
    pub(crate) tier_epoch_anchor: Option<[u8; 32]>,
}

pub(crate) fn read_root_manifest(
    bytes: &[u8],
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<OfflineRootManifestFacts, OfflineIntegrityOutcome> {
    let frame = read_durable_frame(
        bytes,
        expected_root_manifest_bytes(bytes),
        ROOT_MANIFEST_KIND,
        ROOT_MANIFEST_INTEGRITY_DECLARATION,
        counters,
    )?;
    counters.root_manifest_payload_decoder_entries += 1;
    validate_reserved_and_flags(frame.payload)?;
    let generation = read_u64(frame.payload, 0);
    if generation == 0 || frame.identity != generation {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::ScopeMismatch,
            48,
            8,
            OfflinePhysicalFormatField::ManifestGeneration,
        ));
    }
    let tree_identity = read_u64(frame.payload, 8);
    let capacity = read_u16(frame.payload, 16);
    let page_bytes = read_u32(&frame.format, 2) as usize;
    let maximum_capacity = (page_bytes - 48 - 24) / 88;
    let record_count = read_u64(frame.payload, 24);
    let next_block = read_u64(frame.payload, 32);
    let free_space_checksum = read_u32(frame.payload, 152);
    let next_segment_block = read_u64(frame.payload, 224);
    if tree_identity == 0
        || capacity < 2
        || usize::from(capacity) > maximum_capacity
        || next_block == 0
        || next_segment_block == 0
        || free_space_checksum == 0
    {
        return Err(malformed_manifest());
    }
    validate_manifest_shape(
        frame.payload,
        generation,
        record_count,
        capacity,
        next_block,
        next_segment_block,
    )?;
    let anchored = bytes[9] == 9;
    if frame.payload.len() >= 496 {
        validate_derived_directory_binding(frame.payload, generation, !anchored)?;
    }
    if frame.payload.len() >= 528 {
        validate_quarantine_binding(frame.payload, anchored)?;
    }
    let tier_epoch_anchor = if anchored {
        let anchor: [u8; 32] = frame.payload[528..560].try_into().expect("schema9 width");
        if anchor == [0; 32] {
            return Err(malformed_manifest());
        }
        Some(anchor)
    } else {
        None
    };
    let mut payload = [0; 560];
    payload[..frame.payload.len()].copy_from_slice(frame.payload);
    Ok(OfflineRootManifestFacts {
        generation,
        format: frame.format,
        payload,
        encoded_bytes: bytes.len(),
        tier_epoch_anchor,
    })
}

fn validate_quarantine_binding(
    payload: &[u8],
    allow_absent: bool,
) -> Result<(), OfflineIntegrityOutcome> {
    let routing = (payload[40] == 1).then(|| &payload[48..120]);
    let record = &payload[504..528];
    if payload[497..504] != [0; 7] {
        return Err(pointer_damage(545, 7));
    }
    match payload[496] {
        0 if allow_absent && record == [0; 24] => Ok(()),
        1 if valid_record_identity(record)
            && routing.is_some_and(|reference| routing_contains(reference, record)) =>
        {
            Ok(())
        }
        _ => Err(pointer_damage(544, 32)),
    }
}

fn validate_derived_directory_binding(
    payload: &[u8],
    generation: u64,
    require_directory: bool,
) -> Result<(), OfflineIntegrityOutcome> {
    let routing = (payload[40] == 1).then(|| &payload[48..120]);
    let latest = &payload[336..400];
    match payload[401] {
        0 if latest == [0; 64] => {}
        1 if valid_record_identity(&latest[..24])
            && read_u64(latest, 24) != 0
            && read_u64(latest, 24) <= generation
            && routing.is_some_and(|reference| routing_contains(reference, &latest[..24])) => {}
        _ => return Err(pointer_damage(384, 64)),
    }
    if payload[402..408] != [0; 6] {
        return Err(pointer_damage(450, 6));
    }
    if require_directory && payload[400] == 0 && payload[401] == 0 {
        return Err(pointer_damage(448, 2));
    }
    let binding = &payload[408..496];
    match payload[400] {
        0 if binding == [0; 88] => Ok(()),
        1 if valid_record_identity(&binding[..24])
            && routing.is_some_and(|reference| routing_contains(reference, &binding[..24]))
            && (binding[24..88] == [0; 64]
                || (valid_record_identity(&binding[24..48])
                    && read_u64(binding, 48) != 0
                    && read_u64(binding, 48) <= generation
                    && routing.is_some_and(|reference| {
                        routing_contains(reference, &binding[24..48])
                    }))) =>
        {
            Ok(())
        }
        _ => Err(pointer_damage(448, 96)),
    }
}

fn validate_reserved_and_flags(payload: &[u8]) -> Result<(), OfflineIntegrityOutcome> {
    for (start, end) in [
        (18, 24),
        (41, 48),
        (121, 128),
        (156, 160),
        (161, 168),
        (233, 240),
        (313, 320),
    ] {
        if payload[start..end].iter().any(|byte| *byte != 0) {
            return Err(damaged_field(
                OfflinePhysicalDamageCause::MalformedPayload,
                (48 + start) as u64,
                (end - start) as u64,
                OfflinePhysicalFormatField::Reserved,
            ));
        }
    }
    for offset in [40, 120, 160, 232, 312] {
        if payload[offset] > 1 {
            return Err(damaged_field(
                OfflinePhysicalDamageCause::Pointer,
                (48 + offset) as u64,
                1,
                OfflinePhysicalFormatField::ManifestPointer,
            ));
        }
    }
    Ok(())
}

fn validate_manifest_shape(
    payload: &[u8],
    generation: u64,
    record_count: u64,
    capacity: u16,
    next_block: u64,
    next_segment_block: u64,
) -> Result<(), OfflineIntegrityOutcome> {
    let routing = (payload[40] == 1).then(|| &payload[48..120]);
    let last_record = (payload[120] == 1).then(|| &payload[128..152]);
    let segment = (payload[160] == 1).then(|| &payload[168..224]);
    let free_space = (payload[232] == 1).then(|| &payload[240..312]);
    let last_segment = (payload[312] == 1).then(|| &payload[320..336]);
    if last_record.is_some() != last_segment.is_some() {
        return Err(pointer_damage(168, 200));
    }
    if record_count == 0 && (routing.is_some() || last_record.is_some()) {
        return Err(pointer_damage(88, 112));
    }
    if record_count == 0 && segment.is_some() {
        return Err(pointer_damage(208, 1));
    }
    if record_count != 0 && routing.is_none() {
        return Err(pointer_damage(88, 1));
    }
    if free_space.is_none() && payload[240..312].iter().any(|byte| *byte != 0) {
        return Err(pointer_damage(288, 72));
    }
    if let Some(reference) = routing {
        let required_level =
            required_tree_level(record_count, capacity).ok_or_else(|| pointer_damage(96, 72))?;
        if !valid_routing_reference(reference, generation, next_block, required_level)
            || last_record.is_some_and(|record| !routing_contains(reference, record))
        {
            return Err(pointer_damage(96, 72));
        }
    }
    let maximum_level = required_tree_level(record_count, capacity).unwrap_or(0);
    if segment.is_some_and(|reference| {
        !valid_segment_reference(reference, generation, next_segment_block, maximum_level)
    }) {
        return Err(pointer_damage(216, 56));
    }
    if let Some(reference) = free_space {
        validate_free_space_reference(reference, generation)?;
    }
    if last_record.is_some_and(|identity| !valid_record_identity(identity))
        || last_segment.is_some_and(|value| read_u64(value, 0) == 0 || read_u64(value, 8) == 0)
    {
        return Err(pointer_damage(176, 192));
    }
    Ok(())
}

fn valid_routing_reference(bytes: &[u8], generation: u64, next: u64, level: u16) -> bool {
    valid_simple_reference(bytes, generation, next)
        && read_u16(bytes, 16) == level
        && valid_record_identity(&bytes[24..48])
        && valid_record_identity(&bytes[48..72])
        && record_identity_le(&bytes[24..48], &bytes[48..72])
}

fn routing_contains(reference: &[u8], record: &[u8]) -> bool {
    record_identity_le(&reference[24..48], record) && record_identity_le(record, &reference[48..72])
}

fn valid_simple_reference(bytes: &[u8], generation: u64, next: u64) -> bool {
    read_u64(bytes, 0) != 0
        && read_u64(bytes, 0) <= generation
        && read_u64(bytes, 8) != 0
        && read_u64(bytes, 8) < next
        && bytes[18..20] == [0, 0]
}

fn valid_segment_reference(bytes: &[u8], generation: u64, next: u64, maximum_level: u16) -> bool {
    valid_simple_reference(bytes, generation, next)
        && read_u16(bytes, 16) <= maximum_level
        && valid_segment_key(&bytes[24..40])
        && valid_segment_key(&bytes[40..56])
        && segment_key(&bytes[24..40]) <= segment_key(&bytes[40..56])
}

fn validate_free_space_reference(
    bytes: &[u8],
    generation: u64,
) -> Result<(), OfflineIntegrityOutcome> {
    if read_u64(bytes, 0) == 0 || read_u64(bytes, 0) > generation {
        return Err(pointer_damage(288, 8));
    }
    if read_u64(bytes, 8) == 0 {
        return Err(pointer_damage(296, 8));
    }
    if bytes[18..20] != [0, 0] {
        return Err(pointer_damage(306, 2));
    }
    if !valid_free_space_key(&bytes[24..48]) {
        return Err(pointer_damage(312, 24));
    }
    if !valid_free_space_key(&bytes[48..72])
        || free_space_key(&bytes[24..48]) > free_space_key(&bytes[48..72])
    {
        return Err(pointer_damage(336, 24));
    }
    Ok(())
}

fn valid_record_identity(bytes: &[u8]) -> bool {
    bytes[..16] != [0; 16] && read_u64(bytes, 16) != 0
}

fn record_identity_le(left: &[u8], right: &[u8]) -> bool {
    (&left[..16], read_u64(left, 16)) <= (&right[..16], read_u64(right, 16))
}

fn valid_segment_key(bytes: &[u8]) -> bool {
    read_u64(bytes, 0) != 0 && read_u64(bytes, 8) != 0
}

fn segment_key(bytes: &[u8]) -> (u64, u64) {
    (read_u64(bytes, 0), read_u64(bytes, 8))
}

fn valid_free_space_key(bytes: &[u8]) -> bool {
    matches!(bytes[0], 1 | 2) && bytes[1..8] == [0; 7] && read_u64(bytes, 8) != 0
}

fn free_space_key(bytes: &[u8]) -> (u8, u64, u64) {
    (bytes[0], read_u64(bytes, 8), read_u64(bytes, 16))
}

fn required_tree_level(entries: u64, capacity: u16) -> Option<u16> {
    if entries == 0 || capacity < 2 {
        return None;
    }
    let capacity = u64::from(capacity);
    let mut nodes = entries.div_ceil(capacity);
    let mut level = 0_u16;
    while nodes > 1 {
        nodes = nodes.div_ceil(capacity);
        level = level.checked_add(1)?;
    }
    Some(level)
}

fn malformed_manifest() -> OfflineIntegrityOutcome {
    damaged_field(
        OfflinePhysicalDamageCause::MalformedPayload,
        48,
        336,
        OfflinePhysicalFormatField::ManifestPointer,
    )
}

fn pointer_damage(offset: u64, length: u64) -> OfflineIntegrityOutcome {
    OfflineIntegrityOutcome::Damaged(super::super::OfflinePhysicalDamageLocalization::new(
        OfflinePhysicalDamageCause::Pointer,
        Some((offset, length)),
        Some(OfflinePhysicalFormatField::ManifestPointer),
        super::super::OfflinePhysicalBlastRadius::ReachableRootSubtree,
    ))
}
