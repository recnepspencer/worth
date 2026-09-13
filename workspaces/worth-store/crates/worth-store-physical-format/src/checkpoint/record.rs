use crate::record_framing::crc32c;

const MAGIC: [u8; 8] = *b"WCP7REC\0";
const SCHEMA: u8 = 1;
const PREFIX_BYTES: usize = 16;
const CHECKSUM_BYTES: usize = 4;

pub(super) const HEADER_KIND: u8 = 1;
pub(super) const DIRTY_BASIS_KIND: u8 = 2;
pub(super) const BINDING_COMPACTION_HEADER_KIND: u8 = 3;
pub(super) const BINDING_RECORD_KIND: u8 = 4;
pub(super) const FOOTER_KIND: u8 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointStreamDecodeDenial {
    Truncated,
    WrongMagic,
    UnsupportedSchema(u8),
    WrongRecordKind { expected: u8, actual: u8 },
    ReservedFieldNonZero,
    LengthMismatch,
    IntegrityMismatch,
    InvalidIdentity,
    InvalidWalRange,
    InvalidCapturePosture(u8),
    InvalidSecurityBindingPresence(u8),
    AbsentSecurityBindingResidue { payload_offset: u8 },
    InvalidSecurityPolicyIdentity,
    InvalidSecurityRetention,
    SecurityBindingDigestMismatch,
    InvalidArtifactKind(u8),
    InvalidCoordinate,
    InvalidBindingCompactionHeader,
    EmptyBindingRecord,
    BindingRecordTooLarge,
    SourceIdentityMismatch,
    RecordCountMismatch,
    RecordByteCountMismatch,
    BindingCompactionMismatch,
    AggregateDigestMismatch,
}

pub(super) fn encode_record(kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut record = vec![0; PREFIX_BYTES + payload.len() + CHECKSUM_BYTES];
    record[..8].copy_from_slice(&MAGIC);
    record[8] = SCHEMA;
    record[9] = kind;
    record[12..16].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    record[PREFIX_BYTES..PREFIX_BYTES + payload.len()].copy_from_slice(payload);
    let checksum = crc32c::checksum(&[&record[..PREFIX_BYTES], payload]);
    record[PREFIX_BYTES + payload.len()..].copy_from_slice(&checksum.to_le_bytes());
    record
}

pub(super) fn decode_record(
    record: &[u8],
    expected_kind: u8,
    expected_payload_bytes: usize,
) -> Result<&[u8], CheckpointStreamDecodeDenial> {
    if record.len() < PREFIX_BYTES + CHECKSUM_BYTES {
        return Err(CheckpointStreamDecodeDenial::Truncated);
    }
    if record[..8] != MAGIC {
        return Err(CheckpointStreamDecodeDenial::WrongMagic);
    }
    if record[8] != SCHEMA {
        return Err(CheckpointStreamDecodeDenial::UnsupportedSchema(record[8]));
    }
    if record[9] != expected_kind {
        return Err(CheckpointStreamDecodeDenial::WrongRecordKind {
            expected: expected_kind,
            actual: record[9],
        });
    }
    if record[10..12] != [0; 2] {
        return Err(CheckpointStreamDecodeDenial::ReservedFieldNonZero);
    }
    let payload_bytes = u32::from_le_bytes(record[12..16].try_into().unwrap()) as usize;
    if payload_bytes != expected_payload_bytes
        || record.len() != PREFIX_BYTES + payload_bytes + CHECKSUM_BYTES
    {
        return Err(CheckpointStreamDecodeDenial::LengthMismatch);
    }
    let payload = &record[PREFIX_BYTES..PREFIX_BYTES + payload_bytes];
    let stored = u32::from_le_bytes(record[PREFIX_BYTES + payload_bytes..].try_into().unwrap());
    let actual = crc32c::checksum(&[&record[..PREFIX_BYTES], payload]);
    if stored != actual {
        return Err(CheckpointStreamDecodeDenial::IntegrityMismatch);
    }
    Ok(payload)
}

pub(super) fn decode_bounded_record(
    record: &[u8],
    expected_kind: u8,
    maximum_payload_bytes: usize,
) -> Result<&[u8], CheckpointStreamDecodeDenial> {
    if record.len() < PREFIX_BYTES + CHECKSUM_BYTES {
        return Err(CheckpointStreamDecodeDenial::Truncated);
    }
    if record[..8] != MAGIC {
        return Err(CheckpointStreamDecodeDenial::WrongMagic);
    }
    if record[8] != SCHEMA {
        return Err(CheckpointStreamDecodeDenial::UnsupportedSchema(record[8]));
    }
    if record[9] != expected_kind {
        return Err(CheckpointStreamDecodeDenial::WrongRecordKind {
            expected: expected_kind,
            actual: record[9],
        });
    }
    if record[10..12] != [0; 2] {
        return Err(CheckpointStreamDecodeDenial::ReservedFieldNonZero);
    }
    let payload_bytes = u32::from_le_bytes(record[12..16].try_into().unwrap()) as usize;
    if payload_bytes == 0 {
        return Err(CheckpointStreamDecodeDenial::EmptyBindingRecord);
    }
    if payload_bytes > maximum_payload_bytes {
        return Err(CheckpointStreamDecodeDenial::BindingRecordTooLarge);
    }
    if record.len() != PREFIX_BYTES + payload_bytes + CHECKSUM_BYTES {
        return Err(CheckpointStreamDecodeDenial::LengthMismatch);
    }
    let payload = &record[PREFIX_BYTES..PREFIX_BYTES + payload_bytes];
    let stored = u32::from_le_bytes(record[PREFIX_BYTES + payload_bytes..].try_into().unwrap());
    let actual = crc32c::checksum(&[&record[..PREFIX_BYTES], payload]);
    if stored != actual {
        return Err(CheckpointStreamDecodeDenial::IntegrityMismatch);
    }
    Ok(payload)
}

pub(super) fn decode_bounded_record_frame_bytes(
    prefix: &[u8],
    expected_kind: u8,
    maximum_payload_bytes: usize,
) -> Result<usize, CheckpointStreamDecodeDenial> {
    if prefix.len() < PREFIX_BYTES {
        return Err(CheckpointStreamDecodeDenial::Truncated);
    }
    if prefix.len() != PREFIX_BYTES {
        return Err(CheckpointStreamDecodeDenial::LengthMismatch);
    }
    if prefix[..8] != MAGIC {
        return Err(CheckpointStreamDecodeDenial::WrongMagic);
    }
    if prefix[8] != SCHEMA {
        return Err(CheckpointStreamDecodeDenial::UnsupportedSchema(prefix[8]));
    }
    if prefix[9] != expected_kind {
        return Err(CheckpointStreamDecodeDenial::WrongRecordKind {
            expected: expected_kind,
            actual: prefix[9],
        });
    }
    if prefix[10..12] != [0; 2] {
        return Err(CheckpointStreamDecodeDenial::ReservedFieldNonZero);
    }
    let payload_bytes = u32::from_le_bytes(prefix[12..16].try_into().unwrap()) as usize;
    if payload_bytes == 0 {
        return Err(CheckpointStreamDecodeDenial::EmptyBindingRecord);
    }
    if payload_bytes > maximum_payload_bytes {
        return Err(CheckpointStreamDecodeDenial::BindingRecordTooLarge);
    }
    PREFIX_BYTES
        .checked_add(payload_bytes)
        .and_then(|bytes| bytes.checked_add(CHECKSUM_BYTES))
        .ok_or(CheckpointStreamDecodeDenial::LengthMismatch)
}

pub(super) fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
