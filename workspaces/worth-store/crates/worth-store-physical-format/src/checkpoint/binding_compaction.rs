use super::record::{read_u64, CheckpointStreamDecodeDenial};

pub const PHYSICAL_MUTATION_BINDING_COMPACTION_RECORD_DOMAIN: &[u8] =
    b"store.physical.mutation-binding-compaction-record.v1";

pub(super) const BINDING_COMPACTION_HEADER_PAYLOAD_BYTES: usize = 16;
/// A completed batch carries every returned record identity in its terminal
/// fate. A batch admits at most `u16::MAX` records; each identity occupies a
/// length-tagged 16-byte epoch and an 8-byte ordinal (32 bytes). Reserve the
/// fixed binding, group, breadth and observation fields separately.
pub const MAX_CHECKPOINT_BINDING_RECORD_BYTES: usize = 4 * 1024 + u16::MAX as usize * 32;
pub const CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES: usize = 36;
pub const CHECKPOINT_BINDING_RECORD_PREFIX_BYTES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointBindingRecordFrameLength(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointBindingCompactionHeader {
    generation: u64,
    wal_cutoff_lsn_exclusive: u64,
}

impl CheckpointBindingCompactionHeader {
    pub fn decode_record(record: &[u8]) -> Result<Self, CheckpointStreamDecodeDenial> {
        let payload = super::record::decode_record(
            record,
            super::record::BINDING_COMPACTION_HEADER_KIND,
            BINDING_COMPACTION_HEADER_PAYLOAD_BYTES,
        )?;
        decode_binding_compaction_header(payload)
    }

    pub const fn new(generation: u64, wal_cutoff_lsn_exclusive: u64) -> Option<Self> {
        if generation == 0 || wal_cutoff_lsn_exclusive == 0 {
            return None;
        }
        Some(Self {
            generation,
            wal_cutoff_lsn_exclusive,
        })
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn wal_cutoff_lsn_exclusive(self) -> u64 {
        self.wal_cutoff_lsn_exclusive
    }
}

pub fn decode_checkpoint_binding_record(
    record: &[u8],
) -> Result<&[u8], CheckpointStreamDecodeDenial> {
    super::record::decode_bounded_record(
        record,
        super::record::BINDING_RECORD_KIND,
        MAX_CHECKPOINT_BINDING_RECORD_BYTES,
    )
}

impl CheckpointBindingRecordFrameLength {
    pub fn decode_prefix(prefix: &[u8]) -> Result<Self, CheckpointStreamDecodeDenial> {
        super::record::decode_bounded_record_frame_bytes(
            prefix,
            super::record::BINDING_RECORD_KIND,
            MAX_CHECKPOINT_BINDING_RECORD_BYTES,
        )
        .map(Self)
    }

    pub const fn encoded_bytes(self) -> usize {
        self.0
    }
}

pub(super) fn encode_binding_compaction_header(
    header: CheckpointBindingCompactionHeader,
) -> [u8; BINDING_COMPACTION_HEADER_PAYLOAD_BYTES] {
    let mut payload = [0; BINDING_COMPACTION_HEADER_PAYLOAD_BYTES];
    payload[..8].copy_from_slice(&header.generation.to_le_bytes());
    payload[8..].copy_from_slice(&header.wal_cutoff_lsn_exclusive.to_le_bytes());
    payload
}

pub(super) fn decode_binding_compaction_header(
    payload: &[u8],
) -> Result<CheckpointBindingCompactionHeader, CheckpointStreamDecodeDenial> {
    CheckpointBindingCompactionHeader::new(read_u64(payload, 0), read_u64(payload, 8))
        .ok_or(CheckpointStreamDecodeDenial::InvalidBindingCompactionHeader)
}
