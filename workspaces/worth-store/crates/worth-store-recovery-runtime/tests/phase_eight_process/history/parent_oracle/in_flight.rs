use sha2::{Digest, Sha256};

#[path = "in_flight/record_binding.rs"]
mod record_binding;
pub(crate) use record_binding::{require_bound_record, require_bound_records};
#[path = "in_flight/checkpoint_binding.rs"]
mod checkpoint_binding;
pub(crate) use checkpoint_binding::scan_checkpoint_binding;

const WAL_HEADER_BYTES: usize = 116;
const WAL_FOOTER_BYTES: usize = 32;
const CHECKPOINT_PREFIX_BYTES: usize = 16;
const COMPACTION_DOMAIN: &[u8] =
    worth_store_physical_format::PHYSICAL_MUTATION_BINDING_COMPACTION_RECORD_DOMAIN;
const ATTEMPT_DOMAIN: &[u8] = b"store.physical.mutation-attempt-binding.v1";
const KEY_DOMAIN: &[u8] = b"store.physical.mutation.idempotency-key.v1";

pub(crate) fn classify(
    files: &[(String, Vec<u8>)],
    identity: &[u8],
    payload: &[u8],
) -> Result<(bool, bool), String> {
    let mut identity_present = false;
    let mut payload_present = false;
    for (path, bytes) in files {
        let (identity, payload) = if bytes.starts_with(b"WORTHWAL") {
            scan_wal(bytes, identity, payload)?
        } else if bytes.starts_with(b"WCP7REC\0") {
            scan_checkpoint(bytes, identity)?
        } else {
            (false, record_payload_present(path, bytes, payload)?)
        };
        identity_present |= identity;
        payload_present |= payload;
    }
    Ok((identity_present, payload_present))
}

fn record_payload_present(path: &str, bytes: &[u8], payload: &[u8]) -> Result<bool, String> {
    if !is_record_artifact(path) {
        return Ok(false);
    }
    let mut offset = 0;
    let mut found = false;
    while offset < bytes.len() {
        let frame = super::canonical_membership_frame::frame_at(bytes, offset)
            .ok_or_else(|| format!("semantic record oracle found a malformed frame: {path}"))?;
        found |= contains_bytes(frame.payload, payload);
        offset = offset
            .checked_add(super::canonical_membership_frame::frame_total(
                bytes, offset,
            )?)
            .ok_or_else(|| "semantic record oracle frame offset overflowed".to_owned())?;
    }
    Ok(found)
}

fn is_record_artifact(path: &str) -> bool {
    (path.starts_with("families/records/segments/") && path.ends_with(".pages"))
        || (path.starts_with("families/records/extents/") && path.ends_with(".data"))
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn scan_wal(bytes: &[u8], identity: &[u8], payload: &[u8]) -> Result<(bool, bool), String> {
    let mut offset = 0;
    let mut found_identity = false;
    let mut found_payload = false;
    while offset < bytes.len() {
        let header = bytes
            .get(offset..offset + WAL_HEADER_BYTES)
            .ok_or_else(|| "semantic WAL oracle found a truncated header".to_owned())?;
        if &header[..8] != b"WORTHWAL" || read_u16(header, 8) != Some(1) {
            return Err("semantic WAL oracle found an invalid frame header".to_owned());
        }
        let payload_bytes = read_u64(header, 44)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| "semantic WAL oracle found an invalid payload length".to_owned())?;
        let total = WAL_HEADER_BYTES
            .checked_add(payload_bytes)
            .and_then(|value| value.checked_add(WAL_FOOTER_BYTES))
            .ok_or_else(|| "semantic WAL oracle frame length overflowed".to_owned())?;
        let frame = bytes
            .get(offset..offset + total)
            .ok_or_else(|| "semantic WAL oracle found a truncated frame".to_owned())?;
        let frame_payload = &frame[WAL_HEADER_BYTES..WAL_HEADER_BYTES + payload_bytes];
        if Sha256::digest(frame_payload)[..] != header[84..116]
            || Sha256::digest(&frame[..WAL_HEADER_BYTES + payload_bytes])[..]
                != frame[WAL_HEADER_BYTES + payload_bytes..]
        {
            return Err("semantic WAL oracle rejected a frame checksum".to_owned());
        }
        let (binding, remaining) = take_field(frame_payload)?;
        let (redo, remaining) = take_field(remaining)?;
        if !remaining.is_empty() {
            return Err("semantic WAL oracle found trailing member payload".to_owned());
        }
        found_identity |= binding_matches(binding, identity, Some(redo));
        found_payload |= binding_matches(binding, identity, Some(payload)) && redo == payload;
        offset += total;
    }
    Ok((found_identity, found_payload))
}

fn scan_checkpoint(bytes: &[u8], identity: &[u8]) -> Result<(bool, bool), String> {
    let found_identity = checkpoint_binding::scan_identity(bytes, identity)?;
    Ok((found_identity, false))
}

fn binding_record_matches(bytes: &[u8], identity: &[u8]) -> bool {
    let mut cursor = Cursor::new(bytes);
    if cursor.field() != Some(COMPACTION_DOMAIN) {
        return false;
    }
    match cursor.byte() {
        Some(1..=3) => basis_matches(&mut cursor, identity),
        Some(4) => cursor
            .field()
            .is_some_and(|binding| binding_matches(binding, identity, None)),
        _ => false,
    }
}

fn binding_matches(bytes: &[u8], identity: &[u8], redo: Option<&[u8]>) -> bool {
    binding_redo_digest(bytes, identity, redo).is_some()
}

fn binding_redo_digest(bytes: &[u8], identity: &[u8], redo: Option<&[u8]>) -> Option<[u8; 32]> {
    let mut cursor = Cursor::new(bytes);
    if cursor.field() != Some(ATTEMPT_DOMAIN) || cursor.field() != Some(identity) {
        return None;
    }
    let store = cursor.array_field(16)?;
    let policy = cursor.array_field(32)?;
    let issuance = cursor.u64()?;
    let expiry = cursor.u64()?;
    let material = cursor.array_field(32)?;
    let fingerprint = cursor.array_field(32)?;
    let mutation_store = cursor.array_field(16)?;
    let runtime = cursor.u64()?;
    let lifecycle = cursor.u64()?;
    let operation = cursor.u64()?;
    if store == [0; 16]
        || policy == [0; 32]
        || material == [0; 32]
        || fingerprint == [0; 32]
        || mutation_store != store
        || issuance >= expiry
        || runtime == 0
        || lifecycle == 0
        || operation == 0
    {
        return None;
    }
    let mut key = Sha256::new();
    write_field(&mut key, KEY_DOMAIN);
    key.update(store);
    key.update(policy);
    key.update(issuance.to_le_bytes());
    key.update(expiry.to_le_bytes());
    key.update(material);
    if key.finalize()[..] != identity[..] {
        return None;
    }
    if cursor.array_field(32).is_none()
        || cursor.u32().is_none()
        || cursor.u32().is_none()
        || cursor.array_field(32).is_none()
        || cursor.array_field(32).is_none()
    {
        return None;
    }
    let start = cursor.u64()?;
    let end = cursor.u64()?;
    let redo_digest = cursor.array_field(32)?;
    if !cursor.is_empty() || start >= end {
        return None;
    }
    if let Some(redo) = redo {
        if Sha256::digest(redo)[..] != redo_digest[..] {
            return None;
        }
    }
    redo_digest.try_into().ok()
}

fn basis_matches(cursor: &mut Cursor<'_>, identity: &[u8]) -> bool {
    let Some(encoded_identity) = cursor.field() else {
        return false;
    };
    if encoded_identity != identity {
        return false;
    }
    let Some(store) = cursor.array_field(16) else {
        return false;
    };
    let Some(policy) = cursor.array_field(32) else {
        return false;
    };
    let Some(issuance) = cursor.u64() else {
        return false;
    };
    let Some(expiry) = cursor.u64() else {
        return false;
    };
    let Some(material) = cursor.array_field(32) else {
        return false;
    };
    let Some(fingerprint) = cursor.array_field(32) else {
        return false;
    };
    let Some(mutation_store) = cursor.array_field(16) else {
        return false;
    };
    let Some(runtime) = cursor.u64() else {
        return false;
    };
    let Some(lifecycle) = cursor.u64() else {
        return false;
    };
    let Some(operation) = cursor.u64() else {
        return false;
    };
    let mut key = Sha256::new();
    write_field(&mut key, KEY_DOMAIN);
    key.update(store);
    key.update(policy);
    key.update(issuance.to_le_bytes());
    key.update(expiry.to_le_bytes());
    key.update(material);
    key.finalize()[..] == identity[..]
        && store != [0; 16]
        && policy != [0; 32]
        && fingerprint != [0; 32]
        && mutation_store == store
        && issuance < expiry
        && runtime != 0
        && lifecycle != 0
        && operation != 0
}

struct Cursor<'bytes> {
    remaining: &'bytes [u8],
}

impl<'bytes> Cursor<'bytes> {
    const fn new(bytes: &'bytes [u8]) -> Self {
        Self { remaining: bytes }
    }

    fn field(&mut self) -> Option<&'bytes [u8]> {
        let length = usize::try_from(self.u64()?).ok()?;
        let (field, remaining) = self.remaining.split_at_checked(length)?;
        self.remaining = remaining;
        Some(field)
    }

    fn array_field(&mut self, expected: usize) -> Option<&'bytes [u8]> {
        let field = self.field()?;
        (field.len() == expected).then_some(field)
    }

    fn u32(&mut self) -> Option<u32> {
        let (value, remaining) = self.remaining.split_at_checked(4)?;
        self.remaining = remaining;
        Some(u32::from_le_bytes(value.try_into().ok()?))
    }

    fn u64(&mut self) -> Option<u64> {
        let (value, remaining) = self.remaining.split_at_checked(8)?;
        self.remaining = remaining;
        Some(u64::from_le_bytes(value.try_into().ok()?))
    }

    fn byte(&mut self) -> Option<u8> {
        let (value, remaining) = self.remaining.split_at_checked(1)?;
        self.remaining = remaining;
        value.first().copied()
    }

    const fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }
}

fn take_field(bytes: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let length = read_u64(bytes, 0)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| "semantic WAL oracle found an invalid field length".to_owned())?;
    let end = 8_usize
        .checked_add(length)
        .ok_or_else(|| "semantic WAL oracle field length overflowed".to_owned())?;
    let field = bytes
        .get(8..end)
        .ok_or_else(|| "semantic WAL oracle found a truncated field".to_owned())?;
    Ok((field, &bytes[end..]))
}

fn write_field(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_le_bytes());
    digest.update(bytes);
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    bytes
        .get(offset..offset + 2)?
        .try_into()
        .ok()
        .map(u16::from_le_bytes)
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    bytes
        .get(offset..offset + 4)?
        .try_into()
        .ok()
        .map(u32::from_le_bytes)
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    bytes
        .get(offset..offset + 8)?
        .try_into()
        .ok()
        .map(u64::from_le_bytes)
}

fn crc32c(bytes: &[u8]) -> u32 {
    let mut value = !0_u32;
    for byte in bytes {
        value ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(value & 1);
            value = (value >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !value
}
