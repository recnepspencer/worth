pub(crate) use std::collections::BTreeMap;

pub(crate) use {sha2::Digest, sha2::Sha256};

pub(crate) use crate::phase_eight_process::history::parent_oracle::canonical_membership::ExpectedCanonicalRecord;
pub(crate) use crate::phase_eight_process::history::parent_oracle::canonical_membership_placement::RecordIdentity;

pub(crate) const REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";
pub(crate) const CURRENT_PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v16";

pub(crate) use {crate::phase_eight_process::history::parent_oracle::in_flight::record_binding::operation::inspect_operation};

pub(crate) use {crate::phase_eight_process::history::parent_oracle::in_flight::record_binding::projection_wire::classified_route_metadata, crate::phase_eight_process::history::parent_oracle::in_flight::record_binding::projection_wire::Cursor};

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
    let selected =
        crate::phase_eight_process::history::parent_oracle::selected_basis::select(files)?;
    let mut matches = 0;
    for (_, bytes) in selected {
        if bytes.starts_with(b"WORTHWAL") {
            matches += scan_wal(&bytes, idempotency, record, payload, expected_redo_digest)?;
        } else if bytes.starts_with(b"WCP7REC\0") {
            matches +=
                crate::phase_eight_process::history::parent_oracle::in_flight::checkpoint_binding::scan_checkpoint_binding(&bytes, idempotency, record, expected_redo_digest)?;
        }
    }
    match matches {
        1 => Ok(()),
        0 => Err("parent oracle found no selected binding for the operation".to_owned()),
        _ => Err("parent oracle found ambiguous selected bindings for the operation".to_owned()),
    }
}

pub(crate) fn scan_wal(
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
            .get(offset..offset + crate::history::parent_oracle::in_flight::WAL_HEADER_BYTES)
            .ok_or_else(|| "parent oracle found a truncated WAL header".to_owned())?;
        let payload_bytes = crate::history::parent_oracle::in_flight::read_u64(header, 44)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| "parent oracle found an invalid WAL payload length".to_owned())?;
        let total = crate::history::parent_oracle::in_flight::WAL_HEADER_BYTES
            .checked_add(payload_bytes)
            .and_then(|value| {
                value.checked_add(crate::history::parent_oracle::in_flight::WAL_FOOTER_BYTES)
            })
            .ok_or_else(|| "parent oracle WAL frame length overflowed".to_owned())?;
        let frame = bytes
            .get(offset..offset + total)
            .ok_or_else(|| "parent oracle found a truncated WAL frame".to_owned())?;
        let frame_payload = &frame[crate::history::parent_oracle::in_flight::WAL_HEADER_BYTES
            ..crate::history::parent_oracle::in_flight::WAL_HEADER_BYTES + payload_bytes];
        if sha2::Sha256::digest(frame_payload)[..] != header[84..116]
            || sha2::Sha256::digest(
                &frame
                    [..crate::history::parent_oracle::in_flight::WAL_HEADER_BYTES + payload_bytes],
            )[..]
                != frame
                    [crate::history::parent_oracle::in_flight::WAL_HEADER_BYTES + payload_bytes..]
        {
            return Err("parent oracle rejected a WAL frame checksum".to_owned());
        }
        let (binding, remaining) =
            crate::history::parent_oracle::in_flight::take_field(frame_payload)?;
        let (redo, remaining) = crate::history::parent_oracle::in_flight::take_field(remaining)?;
        if !remaining.is_empty() {
            return Err("parent oracle found trailing WAL member payload".to_owned());
        }
        if crate::history::parent_oracle::in_flight::binding_matches(
            binding,
            idempotency,
            Some(redo),
        ) && redo_contains_record(redo, record, payload)?
            && expected_redo_digest.is_none_or(|expected| {
                crate::history::parent_oracle::in_flight::binding_redo_digest(
                    binding,
                    idempotency,
                    Some(redo),
                )
                .is_some_and(|observed| observed == *expected)
            })
        {
            matches += 1;
        }
        offset += total;
    }
    Ok(matches)
}

pub(crate) fn redo_contains_record(
    redo: &[u8],
    expected_record: RecordIdentity,
    expected_payload: &[u8],
) -> Result<bool, String> {
    let mut cursor = crate::phase_eight_process::history::parent_oracle::in_flight::record_binding::projection_wire::Cursor::new(redo);
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

pub(crate) fn projection_contains_record(
    projection: &[u8],
    expected_record: RecordIdentity,
) -> Result<bool, String> {
    let mut cursor = crate::phase_eight_process::history::parent_oracle::in_flight::record_binding::projection_wire::Cursor::new(projection);
    if cursor.field()? != CURRENT_PROJECTION_DOMAIN {
        return Ok(false);
    }
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
    let placement_count = cursor.u64()?;
    if placement_count == 0 {
        return Ok(false);
    }
    let mut placement_present = false;
    let mut blob_placement_matches = false;
    let mut copy_placement_matches = false;
    for _ in 0..placement_count {
        let mut placement = crate::phase_eight_process::history::parent_oracle::in_flight::record_binding::projection_wire::Cursor::new(cursor.field()?);
        let kind = placement.byte()?;
        let record = placement.raw_record()?;
        placement_present |= record == expected_record;
        blob_placement_matches |= kind == 2 && record == expected_record;
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
        if !classified_route_metadata(placement.take(7)?) {
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
    let blob_binding = inspect_operation(&mut cursor)?;
    if source_copy && blob_binding.is_some() {
        return Ok(false);
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
