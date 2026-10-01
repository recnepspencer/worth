//! Bounded, purpose-tagged checkpoint custody records. Their payload semantics
//! remain owned by the tier and released-drop authorities respectively.

use super::record::{
    decode_bounded_record, decode_bounded_record_frame_bytes, encode_record_schema,
    CheckpointStreamDecodeDenial, CERTIFIED_CHECKPOINT_SCHEMA, RELEASE_CUSTODY_CERTIFICATE_KIND,
    TIER_EPOCH_CERTIFICATE_KIND,
};

pub const MAX_CHECKPOINT_CERTIFICATE_RECORDS: u64 = 64;
pub const MAX_CHECKPOINT_CERTIFICATE_BYTES: u64 = 65_536;
pub const CHECKPOINT_CERTIFICATE_PREFIX_BYTES: usize = 16;
const RECORD_OVERHEAD: usize = 20;
const MAX_PAYLOAD_BYTES: usize = MAX_CHECKPOINT_CERTIFICATE_BYTES as usize - RECORD_OVERHEAD;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointCertificateKind {
    TierEpoch,
    ReleasedDrop,
}

impl CheckpointCertificateKind {
    const fn code(self) -> u8 {
        match self {
            Self::TierEpoch => TIER_EPOCH_CERTIFICATE_KIND,
            Self::ReleasedDrop => RELEASE_CUSTODY_CERTIFICATE_KIND,
        }
    }

    fn from_code(code: u8) -> Result<Self, CheckpointStreamDecodeDenial> {
        match code {
            TIER_EPOCH_CERTIFICATE_KIND => Ok(Self::TierEpoch),
            RELEASE_CUSTODY_CERTIFICATE_KIND => Ok(Self::ReleasedDrop),
            _ => Err(CheckpointStreamDecodeDenial::InvalidArtifactKind(code)),
        }
    }
}

pub fn encode_checkpoint_certificate(
    kind: CheckpointCertificateKind,
    payload: &[u8],
) -> Result<Vec<u8>, CheckpointStreamDecodeDenial> {
    if payload.is_empty() {
        return Err(CheckpointStreamDecodeDenial::EmptyBindingRecord);
    }
    if payload.len() > MAX_PAYLOAD_BYTES {
        return Err(CheckpointStreamDecodeDenial::BindingRecordTooLarge);
    }
    Ok(encode_record_schema(
        CERTIFIED_CHECKPOINT_SCHEMA,
        kind.code(),
        payload,
    ))
}

pub fn checkpoint_certificate_frame_bytes(
    prefix: &[u8],
) -> Result<usize, CheckpointStreamDecodeDenial> {
    if prefix.len() != CHECKPOINT_CERTIFICATE_PREFIX_BYTES {
        return Err(CheckpointStreamDecodeDenial::LengthMismatch);
    }
    if prefix[8] != CERTIFIED_CHECKPOINT_SCHEMA {
        return Err(CheckpointStreamDecodeDenial::UnsupportedSchema(prefix[8]));
    }
    let kind = CheckpointCertificateKind::from_code(prefix[9])?;
    decode_bounded_record_frame_bytes(prefix, kind.code(), MAX_PAYLOAD_BYTES)
}

pub fn decode_checkpoint_certificate(
    record: &[u8],
) -> Result<(CheckpointCertificateKind, &[u8]), CheckpointStreamDecodeDenial> {
    if record.len() < RECORD_OVERHEAD {
        return Err(CheckpointStreamDecodeDenial::Truncated);
    }
    if record[8] != CERTIFIED_CHECKPOINT_SCHEMA {
        return Err(CheckpointStreamDecodeDenial::UnsupportedSchema(record[8]));
    }
    let kind = CheckpointCertificateKind::from_code(record[9])?;
    let payload = decode_bounded_record(record, kind.code(), MAX_PAYLOAD_BYTES)?;
    Ok((kind, payload))
}
