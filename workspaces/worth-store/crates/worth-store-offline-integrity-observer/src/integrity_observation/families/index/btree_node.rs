use super::super::durable_frame::{damaged_field, read_u16, read_u32};
use super::super::physical_fields::{record_key, scope, shape};
use crate::integrity_observation::{
    crc32c::crc32c, OfflineIntegrityObservationCounters, OfflineIntegrityOutcome as Outcome,
    OfflinePhysicalDamageCause as Cause, OfflinePhysicalFormatField as Field,
    OfflineUnsupportedPhysicalVersion, OfflineUnsupportedVersionAxis,
};
use worth_foundational::PhysicalByteRange;

type RecordKey = ([u8; 16], u64);
const HEADER_BYTES: usize = 104;
const DIRECTORY_ENTRY_BYTES: usize = 8;
const MAXIMUM_BYTES: usize = 65_424;

/// Facts from independent offline bytes, not C.5 routing or Store admission.
pub(crate) struct OfflineBTreeNodeFacts {
    pub(crate) level: u8,
    pub(crate) first_child: Option<RecordKey>,
    pub(crate) separator_children: Vec<RecordKey>,
    pub(crate) keys: Vec<Vec<u8>>,
    pub(crate) leaf_records: Vec<RecordKey>,
    pub(crate) previous_sibling: Option<RecordKey>,
    pub(crate) next_sibling: Option<RecordKey>,
    pub(crate) cell_count: u16,
}

pub(crate) fn inspect_btree_node_payload(
    bytes: &[u8],
    expected_family_code: u16,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<OfflineBTreeNodeFacts, Outcome> {
    shape(
        bytes.len() >= HEADER_BYTES && bytes.len() <= MAXIMUM_BYTES,
        0,
        1,
    )?;
    if &bytes[..8] != b"WRC11BTN" {
        return Err(damaged_field(Cause::Framing, 0, 8, Field::Magic));
    }
    if bytes[8] != 1 {
        return Err(Outcome::Unsupported(
            OfflineUnsupportedPhysicalVersion::new(
                OfflineUnsupportedVersionAxis::BTreeNode,
                u64::from(bytes[8]),
                "1",
                PhysicalByteRange::new(8, 1).unwrap(),
            ),
        ));
    }
    let kind = bytes[9];
    let level = bytes[10];
    shape(matches!(kind, 1 | 2) && (kind == 1) == (level == 0), 9, 2)?;
    shape(
        bytes[11] == 0 && bytes[20..24] == [0; 4] && bytes[100..104] == [0; 4],
        11,
        1,
    )?;
    scope(
        expected_family_code != 0 && read_u16(bytes, 12) == expected_family_code,
        12,
        2,
        Field::FamilyKind,
    )?;
    shape(read_u32(bytes, 16) as usize == bytes.len(), 16, 4)?;
    let count = read_u16(bytes, 14);
    let directory_end = HEADER_BYTES + usize::from(count) * DIRECTORY_ENTRY_BYTES;
    shape(count > 0 && directory_end <= bytes.len(), 14, 2)?;
    counters.checksum_calculations += 1;
    let actual = crc32c(&[&bytes[..96], &bytes[100..]]);
    if actual != read_u32(bytes, 96) {
        return Err(damaged_field(
            Cause::ChecksumMismatch,
            96,
            4,
            Field::Checksum,
        ));
    }
    let first_child = optional_record(&bytes[24..48], 24)?;
    shape((kind == 1) == first_child.is_none(), 24, 24)?;
    let previous_sibling = optional_record(&bytes[48..72], 48)?;
    let next_sibling = optional_record(&bytes[72..96], 72)?;
    shape(
        previous_sibling.is_none() || previous_sibling != next_sibling,
        48,
        48,
    )?;
    let mut separator_children = Vec::with_capacity(if kind == 2 { count as usize } else { 0 });
    let mut keys = Vec::with_capacity(count as usize);
    let mut leaf_records = Vec::with_capacity(if kind == 1 && expected_family_code == 1 {
        count as usize
    } else {
        0
    });
    let mut cursor = directory_end;
    let mut previous_key: Option<&[u8]> = None;
    for (index, slot) in bytes[HEADER_BYTES..directory_end]
        .chunks_exact(DIRECTORY_ENTRY_BYTES)
        .enumerate()
    {
        let key_start = read_u16(slot, 0) as usize;
        let key_len = read_u16(slot, 2) as usize;
        let value_start = read_u16(slot, 4) as usize;
        let value_len = read_u16(slot, 6) as usize;
        let value_end = value_start.checked_add(value_len);
        shape(
            key_len > 0
                && value_len > 0
                && key_start == cursor
                && key_start.checked_add(key_len) == Some(value_start)
                && value_end.is_some_and(|end| end <= bytes.len()),
            HEADER_BYTES + index * DIRECTORY_ENTRY_BYTES,
            DIRECTORY_ENTRY_BYTES,
        )?;
        let key = &bytes[key_start..value_start];
        let expected_shape = match expected_family_code {
            1 => Some((24, 24)),
            2 => Some((64, 56)),
            _ => None,
        };
        if let Some((key_width, leaf_width)) = expected_shape {
            if key_len != key_width || (kind == 1 && value_len != leaf_width) {
                return Err(damaged_field(
                    Cause::MalformedPayload,
                    key_start as u64,
                    (key_len + value_len) as u64,
                    Field::PayloadLength,
                ));
            }
        }
        shape(
            previous_key.map_or(true, |prior| prior < key),
            key_start,
            key_len,
        )?;
        previous_key = Some(key);
        keys.push(key.to_vec());
        if kind == 2 {
            shape(value_len == 24, value_start, value_len)?;
            let child = record_key(&bytes[value_start..value_end.unwrap()]).ok_or_else(|| {
                damaged_field(Cause::Pointer, value_start as u64, 24, Field::IdentityField)
            })?;
            separator_children.push(child);
        } else if expected_family_code == 1 {
            leaf_records.push(
                record_key(&bytes[value_start..value_end.unwrap()]).ok_or_else(|| {
                    damaged_field(Cause::Pointer, value_start as u64, 24, Field::IdentityField)
                })?,
            );
        }
        cursor = value_end.unwrap();
    }
    shape(cursor == bytes.len(), cursor.saturating_sub(1), 1)?;
    Ok(OfflineBTreeNodeFacts {
        level,
        first_child,
        separator_children,
        keys,
        leaf_records,
        previous_sibling,
        next_sibling,
        cell_count: count,
    })
}

fn optional_record(bytes: &[u8], offset: u64) -> Result<Option<RecordKey>, Outcome> {
    if bytes == [0; 24] {
        Ok(None)
    } else {
        record_key(bytes)
            .map(Some)
            .ok_or_else(|| damaged_field(Cause::Pointer, offset, 24, Field::IdentityField))
    }
}
