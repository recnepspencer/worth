use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES,
    CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES, CHECKPOINT_DIRTY_FRAME_RECORD_BYTES,
    CHECKPOINT_STREAM_FOOTER_RECORD_BYTES, CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
    MAX_CHECKPOINT_BINDING_RECORD_BYTES,
};

#[path = "checkpoint_stream_integrity/certificate_section.rs"]
mod certificate_section;
#[path = "checkpoint_stream_integrity/record_framing.rs"]
mod record_framing;

use record_framing::{
    fixed_record, prefix_payload_bytes, read_u32, read_u64, supported_schema, CERTIFIED_SCHEMA,
    CHECKSUM_BYTES, PREFIX_BYTES,
};

const HEADER_KIND: u8 = 1;
const DIRTY_KIND: u8 = 2;
const COMPACTION_KIND: u8 = 3;
const BINDING_KIND: u8 = 4;
const FOOTER_KIND: u8 = 5;

pub(super) struct OfflineObservedCheckpointStream {
    store: [u8; 16],
    sequence: u64,
    root_generation: u64,
    root_tree: u64,
    dirty_frontier: u64,
    wal_begin: u64,
    wal_end: u64,
    durable_lsn: u64,
    dirty_records: u64,
    binding_records: u64,
}

impl OfflineObservedCheckpointStream {
    pub(super) const fn store(&self) -> [u8; 16] {
        self.store
    }

    pub(super) const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(super) const fn root_generation(&self) -> u64 {
        self.root_generation
    }

    pub(super) const fn root_tree(&self) -> u64 {
        self.root_tree
    }

    pub(super) const fn dirty_frontier(&self) -> u64 {
        self.dirty_frontier
    }

    pub(super) const fn wal_begin(&self) -> u64 {
        self.wal_begin
    }

    pub(super) const fn wal_end(&self) -> u64 {
        self.wal_end
    }

    pub(super) const fn durable_lsn(&self) -> u64 {
        self.durable_lsn
    }

    pub(super) const fn dirty_records(&self) -> u64 {
        self.dirty_records
    }

    pub(super) const fn binding_records(&self) -> u64 {
        self.binding_records
    }
}

pub(super) fn observe(bytes: &[u8]) -> Option<OfflineObservedCheckpointStream> {
    let schema = supported_schema(bytes.get(..PREFIX_BYTES)?)?;
    let footer_bytes = if schema == CERTIFIED_SCHEMA {
        CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES
    } else {
        CHECKPOINT_STREAM_FOOTER_RECORD_BYTES
    };
    let minimum = CHECKPOINT_STREAM_HEADER_RECORD_BYTES
        .checked_add(CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES)?
        .checked_add(footer_bytes)?;
    if bytes.len() < minimum {
        return None;
    }
    let header_record = bytes.get(..CHECKPOINT_STREAM_HEADER_RECORD_BYTES)?;
    let header = fixed_record(header_record, schema, HEADER_KIND, 144)?;
    let header_fields = header_fields(header)?;

    let footer_offset = bytes.len().checked_sub(footer_bytes)?;
    let footer_record = bytes.get(footer_offset..)?;
    let footer = fixed_record(
        footer_record,
        schema,
        FOOTER_KIND,
        if schema == CERTIFIED_SCHEMA { 184 } else { 136 },
    )?;
    let footer_fields = footer_fields(footer, schema)?;
    if footer_fields.store != header_fields.store
        || footer_fields.sequence != header_fields.sequence
    {
        return None;
    }

    let dirty_count = usize::try_from(footer_fields.dirty_records).ok()?;
    let binding_count = usize::try_from(footer_fields.binding_records).ok()?;
    if dirty_count > bytes.len() / CHECKPOINT_DIRTY_FRAME_RECORD_BYTES
        || binding_count > bytes.len() / CHECKPOINT_BINDING_RECORD_PREFIX_BYTES
    {
        return None;
    }
    let mut offset = CHECKPOINT_STREAM_HEADER_RECORD_BYTES;
    let mut dirty_digest = Sha256::new();
    for _ in 0..dirty_count {
        let end = offset.checked_add(CHECKPOINT_DIRTY_FRAME_RECORD_BYTES)?;
        let record = bytes.get(offset..end)?;
        let payload = fixed_record(record, schema, DIRTY_KIND, 48)?;
        valid_dirty_basis(payload)?;
        dirty_digest.update(record);
        offset = end;
    }
    if offset as u64 != footer_fields.compaction_offset
        || dirty_digest.finalize().as_slice() != footer_fields.dirty_digest
    {
        return None;
    }

    let compaction_end = offset.checked_add(CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES)?;
    let compaction = fixed_record(
        bytes.get(offset..compaction_end)?,
        schema,
        COMPACTION_KIND,
        16,
    )?;
    let compaction_generation = read_u64(compaction, 0)?;
    let compaction_cutoff = read_u64(compaction, 8)?;
    if compaction_generation == 0
        || compaction_cutoff == 0
        || compaction_generation != footer_fields.compaction_generation
        || compaction_cutoff != footer_fields.durable_lsn
    {
        return None;
    }
    offset = compaction_end;

    let binding_start = offset;
    let mut binding_digest = Sha256::new();
    for _ in 0..binding_count {
        let prefix = bytes.get(offset..offset.checked_add(PREFIX_BYTES)?)?;
        let payload_bytes = prefix_payload_bytes(prefix, schema, BINDING_KIND)?;
        if payload_bytes == 0 || payload_bytes > MAX_CHECKPOINT_BINDING_RECORD_BYTES {
            return None;
        }
        let frame_bytes = PREFIX_BYTES
            .checked_add(payload_bytes)?
            .checked_add(CHECKSUM_BYTES)?;
        let end = offset.checked_add(frame_bytes)?;
        if end > footer_offset {
            return None;
        }
        let record = bytes.get(offset..end)?;
        fixed_record(record, schema, BINDING_KIND, payload_bytes)?;
        binding_digest.update(record);
        offset = end;
    }
    if u64::try_from(offset.checked_sub(binding_start)?).ok()? != footer_fields.binding_bytes
        || binding_digest.finalize().as_slice() != footer_fields.binding_digest
    {
        return None;
    }
    if schema == CERTIFIED_SCHEMA {
        certificate_section::observe(
            bytes,
            offset,
            footer_offset,
            footer_fields.certificate_records,
            footer_fields.certificate_bytes,
            footer_fields.certificate_digest,
        )?;
    } else if offset != footer_offset {
        return None;
    }

    Some(OfflineObservedCheckpointStream {
        store: header_fields.store,
        sequence: header_fields.sequence,
        root_generation: header_fields.root_generation,
        root_tree: header_fields.root_tree,
        dirty_frontier: header_fields.dirty_frontier,
        wal_begin: header_fields.wal_begin,
        wal_end: header_fields.wal_end,
        durable_lsn: footer_fields.durable_lsn,
        dirty_records: footer_fields.dirty_records,
        binding_records: footer_fields.binding_records,
    })
}

struct HeaderFields {
    store: [u8; 16],
    sequence: u64,
    wal_begin: u64,
    wal_end: u64,
    root_generation: u64,
    root_tree: u64,
    dirty_frontier: u64,
}

fn header_fields(payload: &[u8]) -> Option<HeaderFields> {
    let store: [u8; 16] = payload.get(..16)?.try_into().ok()?;
    let sequence = read_u64(payload, 16)?;
    let wal_begin = read_u64(payload, 24)?;
    let wal_end = read_u64(payload, 32)?;
    if store == [0; 16]
        || sequence == 0
        || wal_begin >= wal_end
        || payload[64] != 1
        || payload.get(66..72)? != [0; 6]
    {
        return None;
    }
    validate_security_binding(payload, store, sequence, wal_begin, wal_end)?;
    Some(HeaderFields {
        store,
        sequence,
        wal_begin,
        wal_end,
        root_generation: read_u64(payload, 40)?,
        root_tree: read_u64(payload, 48)?,
        dirty_frontier: read_u64(payload, 56)?,
    })
}

fn validate_security_binding(
    payload: &[u8],
    store: [u8; 16],
    sequence: u64,
    wal_begin: u64,
    wal_end: u64,
) -> Option<()> {
    match payload[65] {
        0 if payload.get(72..144)?.iter().all(|byte| *byte == 0) => Some(()),
        1 => {
            let policy = payload.get(72..104)?;
            let retention = read_u64(payload, 104)?;
            if policy.iter().all(|byte| *byte == 0) || retention == 0 {
                return None;
            }
            let mut digest = Sha256::new();
            digest.update(b"worth.store.checkpoint-security-binding.v1");
            digest.update(store);
            digest.update(sequence.to_le_bytes());
            digest.update(wal_begin.to_le_bytes());
            digest.update(wal_end.to_le_bytes());
            digest.update(read_u64(payload, 40)?.to_le_bytes());
            digest.update(read_u64(payload, 48)?.to_le_bytes());
            digest.update(policy);
            digest.update(retention.to_le_bytes());
            (digest.finalize().as_slice() == payload.get(112..144)?).then_some(())
        }
        _ => None,
    }
}

struct FooterFields {
    store: [u8; 16],
    sequence: u64,
    dirty_records: u64,
    dirty_digest: [u8; 32],
    compaction_offset: u64,
    compaction_generation: u64,
    durable_lsn: u64,
    binding_records: u64,
    binding_bytes: u64,
    binding_digest: [u8; 32],
    certificate_records: u64,
    certificate_bytes: u64,
    certificate_digest: [u8; 32],
}

fn footer_fields(payload: &[u8], schema: u8) -> Option<FooterFields> {
    Some(FooterFields {
        store: payload.get(..16)?.try_into().ok()?,
        sequence: read_u64(payload, 16)?,
        dirty_records: read_u64(payload, 24)?,
        dirty_digest: payload.get(32..64)?.try_into().ok()?,
        compaction_offset: read_u64(payload, 64)?,
        compaction_generation: read_u64(payload, 72)?,
        durable_lsn: read_u64(payload, 80)?,
        binding_records: read_u64(payload, 88)?,
        binding_bytes: read_u64(payload, 96)?,
        binding_digest: payload.get(104..136)?.try_into().ok()?,
        certificate_records: if schema == CERTIFIED_SCHEMA {
            read_u64(payload, 136)?
        } else {
            0
        },
        certificate_bytes: if schema == CERTIFIED_SCHEMA {
            read_u64(payload, 144)?
        } else {
            0
        },
        certificate_digest: if schema == CERTIFIED_SCHEMA {
            payload.get(152..184)?.try_into().ok()?
        } else {
            Sha256::digest([]).into()
        },
    })
}

fn valid_dirty_basis(payload: &[u8]) -> Option<()> {
    let kind = payload[0];
    let first = read_u64(payload, 8)?;
    let second = read_u64(payload, 16)?;
    let artifact_valid = match kind {
        1 | 12 | 13 => first == 0 && second == 0,
        2 | 3 | 10 | 14 | 15 => second == 0,
        16 => first != 0 && second == 0,
        17 => first != 0 && second != 0,
        4..=7 | 11 => true,
        _ => false,
    };
    let offset = read_u64(payload, 24)?;
    let length = read_u32(payload, 32)?;
    if !artifact_valid
        || payload.get(1..8)? != [0; 7]
        || payload.get(36..40)? != [0; 4]
        || length == 0
        || offset.checked_add(u64::from(length)).is_none()
    {
        return None;
    }
    Some(())
}

#[cfg(test)]
#[path = "checkpoint_stream_integrity/tests.rs"]
mod tests;
