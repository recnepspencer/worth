use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use super::super::canonical_membership::ExpectedCanonicalRecord;
use super::super::canonical_membership_placement::RecordIdentity;

const REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";
const V5_PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v5";
const V6_PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v6";
const V13_PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v13";

#[path = "record_binding/projection_wire.rs"]
mod projection_wire;
use projection_wire::{classified_route_metadata, Cursor};

pub(crate) fn require_bound_records(
    files: &[(String, Vec<u8>)],
    expected: &BTreeMap<[u8; 32], ExpectedCanonicalRecord>,
) -> Result<(), String> {
    for (idempotency, record) in expected {
        require_bound_record(
            files,
            idempotency,
            RecordIdentity {
                allocation_epoch: record.allocation_epoch,
                ordinal: record.ordinal,
            },
            &record.payload,
            Some(&record.redo_digest),
        )?;
    }
    Ok(())
}

pub(crate) fn require_bound_record(
    files: &[(String, Vec<u8>)],
    idempotency: &[u8],
    record: RecordIdentity,
    payload: &[u8],
    expected_redo_digest: Option<&[u8; 32]>,
) -> Result<(), String> {
    let selected = super::super::selected_basis::select(files)?;
    let mut matches = 0;
    for (_, bytes) in selected {
        if bytes.starts_with(b"WORTHWAL") {
            matches += scan_wal(&bytes, idempotency, record, payload, expected_redo_digest)?;
        } else if bytes.starts_with(b"WCP7REC\0") {
            matches +=
                super::scan_checkpoint_binding(&bytes, idempotency, record, expected_redo_digest)?;
        }
    }
    match matches {
        1 => Ok(()),
        0 => Err("parent oracle found no selected binding for the operation".to_owned()),
        _ => Err("parent oracle found ambiguous selected bindings for the operation".to_owned()),
    }
}

fn scan_wal(
    bytes: &[u8],
    idempotency: &[u8],
    record: RecordIdentity,
    payload: &[u8],
    expected_redo_digest: Option<&[u8; 32]>,
) -> Result<usize, String> {
    let mut offset = 0;
    let mut matches = 0;
    while offset < bytes.len() {
        let header = bytes
            .get(offset..offset + super::WAL_HEADER_BYTES)
            .ok_or_else(|| "parent oracle found a truncated WAL header".to_owned())?;
        let payload_bytes = super::read_u64(header, 44)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| "parent oracle found an invalid WAL payload length".to_owned())?;
        let total = super::WAL_HEADER_BYTES
            .checked_add(payload_bytes)
            .and_then(|value| value.checked_add(super::WAL_FOOTER_BYTES))
            .ok_or_else(|| "parent oracle WAL frame length overflowed".to_owned())?;
        let frame = bytes
            .get(offset..offset + total)
            .ok_or_else(|| "parent oracle found a truncated WAL frame".to_owned())?;
        let frame_payload =
            &frame[super::WAL_HEADER_BYTES..super::WAL_HEADER_BYTES + payload_bytes];
        if Sha256::digest(frame_payload)[..] != header[84..116]
            || Sha256::digest(&frame[..super::WAL_HEADER_BYTES + payload_bytes])[..]
                != frame[super::WAL_HEADER_BYTES + payload_bytes..]
        {
            return Err("parent oracle rejected a WAL frame checksum".to_owned());
        }
        let (binding, remaining) = super::take_field(frame_payload)?;
        let (redo, remaining) = super::take_field(remaining)?;
        if !remaining.is_empty() {
            return Err("parent oracle found trailing WAL member payload".to_owned());
        }
        if super::binding_matches(binding, idempotency, Some(redo))
            && redo_contains_record(redo, record, payload)?
            && expected_redo_digest.is_none_or(|expected| {
                super::binding_redo_digest(binding, idempotency, Some(redo))
                    .is_some_and(|observed| observed == *expected)
            })
        {
            matches += 1;
        }
        offset += total;
    }
    Ok(matches)
}

fn redo_contains_record(
    redo: &[u8],
    expected_record: RecordIdentity,
    expected_payload: &[u8],
) -> Result<bool, String> {
    let mut cursor = Cursor::new(redo);
    if cursor.field()? != REDO_DOMAIN || cursor.u64()? != 1 || cursor.u32()? != 0 {
        return Ok(false);
    }
    if cursor.u64()? == 0 {
        return Ok(false);
    }
    let target_count = cursor.u64()?;
    if target_count == 0 {
        return Ok(false);
    }
    for _ in 0..target_count {
        if cursor.field()?.is_empty() || cursor.take(32)?.iter().all(|byte| *byte == 0) {
            return Ok(false);
        }
    }
    if cursor.field()? != expected_payload {
        return Ok(false);
    }
    let projection = cursor.field()?;
    if !projection_contains_record(projection, expected_record)? || !cursor.is_empty() {
        return Ok(false);
    }
    Ok(true)
}

fn projection_contains_record(
    projection: &[u8],
    expected_record: RecordIdentity,
) -> Result<bool, String> {
    let mut cursor = Cursor::new(projection);
    let domain = cursor.field()?;
    let classified = if domain == V13_PROJECTION_DOMAIN {
        true
    } else if domain == V5_PROJECTION_DOMAIN {
        false
    } else if domain == V6_PROJECTION_DOMAIN {
        false
    } else {
        return Ok(false);
    };
    let source_root = cursor.u64()?;
    if source_root == 0 || cursor.field()?.is_empty() {
        return Ok(false);
    }
    let mut identity_present = false;
    let identity_count = cursor.u64()?;
    for _ in 0..identity_count {
        let record = cursor.record()?;
        identity_present |= record == expected_record;
    }
    let source_copy = match cursor.byte()? {
        0 => {
            let frame_count = cursor.u64()?;
            if frame_count == 0 {
                return Ok(false);
            }
            for _ in 0..frame_count {
                cursor.field()?;
            }
            false
        }
        1 => {
            if cursor.field()?.is_empty() || cursor.u64()? == 0 {
                return Ok(false);
            }
            cursor.take(32)?;
            true
        }
        _ => return Ok(false),
    };
    let blob_binding = if domain != V5_PROJECTION_DOMAIN {
        match blob_semantic_binding(cursor.field()?) {
            Some(Some(_)) if source_copy => return Ok(false),
            Some(binding) => binding,
            None => return Ok(false),
        }
    } else {
        None
    };
    // The only V13 shape admitted here is the ordinary classified append.
    // Derived-directory retirement has its own semantic family and cannot
    // become a record binding through an unexamined extra wire section.
    if classified && cursor.byte()? != 0 {
        return Ok(false);
    }
    let placement_count = cursor.u64()?;
    if placement_count == 0 {
        return Ok(false);
    }
    let mut placement_present = false;
    let mut blob_placement_matches = false;
    let mut copy_placement_matches = false;
    for _ in 0..placement_count {
        let mut placement = Cursor::new(cursor.field()?);
        let kind = placement.byte()?;
        let record = placement.raw_record()?;
        placement_present |= record == expected_record;
        blob_placement_matches |=
            blob_binding.is_some_and(|(bound, _)| kind == 2 && bound == record);
        copy_placement_matches |= kind == 2 && record == expected_record;
        match kind {
            1 => {
                placement.u64()?;
                placement.u64()?;
                placement.u64()?;
                placement.u64()?;
                placement.u16()?;
                placement.u64()?;
                placement.u32()?;
                placement.u64()?;
            }
            2 => {
                let extent = placement.u64()?;
                let generation = placement.u64()?;
                let payload_bytes = placement.u64()?;
                let arena = placement.u64()?;
                let offset = placement.u64()?;
                let length = placement.u64()?;
                if extent == 0
                    || generation == 0
                    || payload_bytes == 0
                    || arena == 0
                    || length == 0
                    || offset.checked_add(length).is_none()
                {
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        }
        if classified && !classified_route_metadata(placement.take(7)?) {
            return Ok(false);
        }
        if !placement.is_empty() {
            return Ok(false);
        }
    }
    let segment_update_count = cursor.u64()?;
    for _ in 0..segment_update_count {
        cursor.field()?;
    }
    let manifest_count = cursor.u64()?;
    for _ in 0..manifest_count {
        cursor.field()?;
    }
    let copy_shape_valid = !source_copy
        || (identity_count == 1
            && placement_count == 1
            && copy_placement_matches
            && segment_update_count == 0
            && manifest_count == 0);
    let blob_binding_valid = blob_binding.is_none_or(|(bound, candidate_root)| {
        identity_count == 1
            && placement_count == 1
            && bound == expected_record
            && blob_placement_matches
            && source_root.checked_add(1) == Some(candidate_root)
    });
    Ok(identity_present
        && placement_present
        && copy_shape_valid
        && blob_binding_valid
        && cursor.is_empty())
}

fn blob_semantic_binding(bytes: &[u8]) -> Option<Option<(RecordIdentity, u64)>> {
    match bytes {
        [0] => Some(None),
        [1 | 2, rest @ ..] if rest.len() == 64 => {
            let mut cursor = Cursor::new(rest);
            let record = cursor.raw_record().ok()?;
            cursor.take(32).ok()?;
            let root = cursor.u64().ok()?;
            (root != 0 && cursor.is_empty()).then_some(Some((record, root)))
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "record_binding/tests.rs"]
mod tests;
