use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use super::super::canonical_membership::ExpectedCanonicalRecord;
use super::super::canonical_membership_placement::RecordIdentity;

const REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";
const PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v5";

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
    if cursor.field()? != PROJECTION_DOMAIN || cursor.u64()? == 0 || cursor.field()?.is_empty() {
        return Ok(false);
    }
    let mut identity_present = false;
    for _ in 0..cursor.u64()? {
        let record = cursor.record()?;
        identity_present |= record == expected_record;
    }
    if cursor.byte()? != 0 {
        return Ok(false);
    }
    let frame_count = cursor.u64()?;
    if frame_count == 0 {
        return Ok(false);
    }
    for _ in 0..frame_count {
        cursor.field()?;
    }
    let placement_count = cursor.u64()?;
    if placement_count == 0 {
        return Ok(false);
    }
    let mut placement_present = false;
    for _ in 0..placement_count {
        let mut placement = Cursor::new(cursor.field()?);
        let kind = placement.byte()?;
        let record = placement.raw_record()?;
        placement_present |= record == expected_record;
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
        if !placement.is_empty() {
            return Ok(false);
        }
    }
    for _ in 0..cursor.u64()? {
        cursor.field()?;
    }
    for _ in 0..cursor.u64()? {
        cursor.field()?;
    }
    Ok(identity_present && placement_present && cursor.is_empty())
}

struct Cursor<'bytes> {
    bytes: &'bytes [u8],
    offset: usize,
}

impl<'bytes> Cursor<'bytes> {
    const fn new(bytes: &'bytes [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn field(&mut self) -> Result<&'bytes [u8], String> {
        let length = usize::try_from(self.u64()?).map_err(|_| "field length overflowed")?;
        let end = self
            .offset
            .checked_add(length)
            .ok_or("field length overflowed")?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or("field is truncated")?;
        self.offset = end;
        Ok(value)
    }

    fn record(&mut self) -> Result<RecordIdentity, String> {
        let bytes = self.field()?;
        Self::decode_record(bytes)
    }

    fn raw_record(&mut self) -> Result<RecordIdentity, String> {
        Self::decode_record(self.take(24)?)
    }

    fn decode_record(bytes: &[u8]) -> Result<RecordIdentity, String> {
        if bytes.len() != 24 {
            return Err("record identity field has the wrong width".to_owned());
        }
        Ok(RecordIdentity {
            allocation_epoch: bytes[..16]
                .try_into()
                .map_err(|_| "record epoch is truncated")?,
            ordinal: u64::from_le_bytes(
                bytes[16..24]
                    .try_into()
                    .map_err(|_| "record ordinal is truncated")?,
            ),
        })
    }

    fn u16(&mut self) -> Result<u16, String> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes(
            bytes.try_into().map_err(|_| "u16 is truncated")?,
        ))
    }

    fn u32(&mut self) -> Result<u32, String> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes(
            bytes.try_into().map_err(|_| "u32 is truncated")?,
        ))
    }

    fn u64(&mut self) -> Result<u64, String> {
        let bytes = self.take(8)?;
        Ok(u64::from_le_bytes(
            bytes.try_into().map_err(|_| "u64 is truncated")?,
        ))
    }

    fn byte(&mut self) -> Result<u8, String> {
        Ok(*self.take(1)?.first().ok_or("byte is truncated")?)
    }

    fn take(&mut self, length: usize) -> Result<&'bytes [u8], String> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or("cursor offset overflowed")?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or("cursor is truncated")?;
        self.offset = end;
        Ok(value)
    }

    const fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{projection_contains_record, RecordIdentity, PROJECTION_DOMAIN};

    #[test]
    fn selected_binding_requires_v5_frame_tag_and_exact_arena_route_fields() {
        let record = RecordIdentity {
            allocation_epoch: [7; 16],
            ordinal: 9,
        };
        let mut encoded = Vec::new();
        field(&mut encoded, PROJECTION_DOMAIN);
        encoded.extend_from_slice(&1_u64.to_le_bytes());
        field(&mut encoded, b"root-state");
        encoded.extend_from_slice(&1_u64.to_le_bytes());
        let mut identity = Vec::from(record.allocation_epoch);
        identity.extend_from_slice(&record.ordinal.to_le_bytes());
        field(&mut encoded, &identity);
        let tag_offset = encoded.len();
        encoded.push(0);
        encoded.extend_from_slice(&1_u64.to_le_bytes());
        field(&mut encoded, b"frame");
        encoded.extend_from_slice(&1_u64.to_le_bytes());
        let mut placement = vec![2];
        placement.extend_from_slice(&identity);
        for value in [3_u64, 4, 1024, 5, 4096, 8192] {
            placement.extend_from_slice(&value.to_le_bytes());
        }
        field(&mut encoded, &placement);
        encoded.extend_from_slice(&0_u64.to_le_bytes());
        encoded.extend_from_slice(&0_u64.to_le_bytes());
        assert!(projection_contains_record(&encoded, record).unwrap());

        encoded[tag_offset] = 1;
        assert!(!projection_contains_record(&encoded, record).unwrap());
        encoded[tag_offset] = 0;
        encoded.truncate(encoded.len() - 8);
        assert!(projection_contains_record(&encoded, record).is_err());
    }

    fn field(target: &mut Vec<u8>, value: &[u8]) {
        target.extend_from_slice(&(value.len() as u64).to_le_bytes());
        target.extend_from_slice(value);
    }
}
