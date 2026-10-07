use super::identity::{decode_identity, encode_identity};
use super::record::{read_u64, CheckpointStreamDecodeDenial};
use super::{CheckpointSelectiveRecordAggregate, PhysicalCheckpointIdentity};

pub(super) const FOOTER_PAYLOAD_BYTES: usize = 136;
pub const CHECKPOINT_STREAM_FOOTER_RECORD_BYTES: usize = 156;
pub(super) const CERTIFIED_FOOTER_PAYLOAD_BYTES: usize = 184;
pub const CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES: usize = 204;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointStreamFooter {
    pub(super) identity: PhysicalCheckpointIdentity,
    pub(super) dirty_record_count: u64,
    pub(super) dirty_records_digest: [u8; 32],
    pub(super) binding_compaction_header_offset: u64,
    pub(super) binding_compaction_generation: u64,
    pub(super) binding_wal_cutoff_lsn_exclusive: u64,
    pub(super) binding_record_count: u64,
    pub(super) binding_record_bytes: u64,
    pub(super) binding_records_digest: [u8; 32],
    pub(super) certificate_record_count: u64,
    pub(super) certificate_record_bytes: u64,
    pub(super) certificate_records_digest: [u8; 32],
}

impl CheckpointStreamFooter {
    pub fn decode_record(record: &[u8]) -> Result<Self, CheckpointStreamDecodeDenial> {
        let certified = record.get(8) == Some(&super::record::CERTIFIED_CHECKPOINT_SCHEMA);
        let payload = super::record::decode_record(
            record,
            super::record::FOOTER_KIND,
            if certified {
                CERTIFIED_FOOTER_PAYLOAD_BYTES
            } else {
                FOOTER_PAYLOAD_BYTES
            },
        )?;
        decode_footer(payload)
    }

    pub const fn identity(self) -> PhysicalCheckpointIdentity {
        self.identity
    }

    pub const fn dirty_record_count(self) -> u64 {
        self.dirty_record_count
    }

    pub const fn dirty_records_digest(self) -> [u8; 32] {
        self.dirty_records_digest
    }

    pub const fn binding_compaction_header_offset(self) -> u64 {
        self.binding_compaction_header_offset
    }

    pub const fn binding_compaction_generation(self) -> u64 {
        self.binding_compaction_generation
    }

    pub const fn binding_wal_cutoff_lsn_exclusive(self) -> u64 {
        self.binding_wal_cutoff_lsn_exclusive
    }

    pub const fn binding_record_count(self) -> u64 {
        self.binding_record_count
    }

    pub const fn binding_record_bytes(self) -> u64 {
        self.binding_record_bytes
    }

    pub const fn binding_records_digest(self) -> [u8; 32] {
        self.binding_records_digest
    }

    pub const fn certificate_record_count(self) -> u64 {
        self.certificate_record_count
    }
    pub const fn certificate_record_bytes(self) -> u64 {
        self.certificate_record_bytes
    }
    pub const fn certificate_records_digest(self) -> [u8; 32] {
        self.certificate_records_digest
    }
}

pub(super) fn encode_footer(footer: CheckpointStreamFooter) -> [u8; FOOTER_PAYLOAD_BYTES] {
    let mut payload = [0; FOOTER_PAYLOAD_BYTES];
    encode_identity(&mut payload[..24], footer.identity);
    payload[24..32].copy_from_slice(&footer.dirty_record_count.to_le_bytes());
    payload[32..64].copy_from_slice(&footer.dirty_records_digest);
    payload[64..72].copy_from_slice(&footer.binding_compaction_header_offset.to_le_bytes());
    payload[72..80].copy_from_slice(&footer.binding_compaction_generation.to_le_bytes());
    payload[80..88].copy_from_slice(&footer.binding_wal_cutoff_lsn_exclusive.to_le_bytes());
    payload[88..96].copy_from_slice(&footer.binding_record_count.to_le_bytes());
    payload[96..104].copy_from_slice(&footer.binding_record_bytes.to_le_bytes());
    payload[104..136].copy_from_slice(&footer.binding_records_digest);
    payload
}

pub(super) fn encode_certified_footer(
    footer: CheckpointStreamFooter,
) -> [u8; CERTIFIED_FOOTER_PAYLOAD_BYTES] {
    let mut payload = [0; CERTIFIED_FOOTER_PAYLOAD_BYTES];
    payload[..FOOTER_PAYLOAD_BYTES].copy_from_slice(&encode_footer(footer));
    payload[136..144].copy_from_slice(&footer.certificate_record_count.to_le_bytes());
    payload[144..152].copy_from_slice(&footer.certificate_record_bytes.to_le_bytes());
    payload[152..184].copy_from_slice(&footer.certificate_records_digest);
    payload
}

pub(super) fn decode_footer(
    payload: &[u8],
) -> Result<CheckpointStreamFooter, CheckpointStreamDecodeDenial> {
    if payload.len() != FOOTER_PAYLOAD_BYTES && payload.len() != CERTIFIED_FOOTER_PAYLOAD_BYTES {
        return Err(CheckpointStreamDecodeDenial::LengthMismatch);
    }
    Ok(CheckpointStreamFooter {
        identity: decode_identity(&payload[..24])?,
        dirty_record_count: read_u64(payload, 24),
        dirty_records_digest: payload[32..64].try_into().unwrap(),
        binding_compaction_header_offset: read_u64(payload, 64),
        binding_compaction_generation: read_u64(payload, 72),
        binding_wal_cutoff_lsn_exclusive: read_u64(payload, 80),
        binding_record_count: read_u64(payload, 88),
        binding_record_bytes: read_u64(payload, 96),
        binding_records_digest: payload[104..136].try_into().unwrap(),
        certificate_record_count: if payload.len() == CERTIFIED_FOOTER_PAYLOAD_BYTES {
            read_u64(payload, 136)
        } else {
            0
        },
        certificate_record_bytes: if payload.len() == CERTIFIED_FOOTER_PAYLOAD_BYTES {
            read_u64(payload, 144)
        } else {
            0
        },
        certificate_records_digest: if payload.len() == CERTIFIED_FOOTER_PAYLOAD_BYTES {
            payload[152..184].try_into().unwrap()
        } else {
            CheckpointSelectiveRecordAggregate::new().summary().digest()
        },
    })
}
