use sha2::{Digest, Sha256};

use super::observer_evidence_accumulation::{
    EvidenceDigestBuilder, RecoveryObserverArtifactEvidence, RecoveryObserverCheckpointObservation,
    RecoveryObserverPageLsnObservation,
};
use super::physical_format;

mod checkpoint_stream_integrity;

pub(super) fn observe_stream(bytes: &[u8]) -> RecoveryObserverArtifactEvidence {
    let Some(stream) = checkpoint_stream_integrity::observe(bytes) else {
        return physical_format::residue(bytes);
    };
    let covered_start = stream.wal_begin();
    let covered_end = stream.wal_end();
    let durable_lsn = stream.durable_lsn();
    if covered_start >= covered_end || durable_lsn < covered_start || durable_lsn > covered_end {
        return physical_format::residue(bytes);
    }
    let mut generation_digest =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    generation_digest.record(&stream.store());
    generation_digest.record(&stream.sequence().to_le_bytes());
    generation_digest.record(&stream.root_generation().to_le_bytes());
    generation_digest.record(&stream.root_tree().to_le_bytes());
    generation_digest.record(&stream.dirty_frontier().to_le_bytes());
    let mut coverage = Vec::with_capacity(40);
    coverage.extend_from_slice(&covered_start.to_le_bytes());
    coverage.extend_from_slice(&covered_end.to_le_bytes());
    coverage.extend_from_slice(&durable_lsn.to_le_bytes());
    coverage.extend_from_slice(&stream.dirty_records().to_le_bytes());
    coverage.extend_from_slice(&stream.binding_records().to_le_bytes());
    let mut checkpoint_digest =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.checkpoint-coverage.v1");
    checkpoint_digest.record(&coverage);
    checkpoint_digest.record(&Sha256::digest(bytes));
    RecoveryObserverArtifactEvidence {
        generation_links: generation_digest.finish(),
        checkpoint: Some(RecoveryObserverCheckpointObservation {
            page_count: stream.dirty_records(),
            covered_lsn: (covered_start, covered_end),
            redo_lsn: covered_start,
            durable_checkpoint_lsn: durable_lsn,
            digest: checkpoint_digest.finish().digest(),
        }),
        ..RecoveryObserverArtifactEvidence::empty()
    }
}

const HEADER_BYTES: usize = 78;
const PAGE_ROW_BYTES: usize = 32;
const FOOTER_BYTES: usize = 32;

pub(super) fn observe(bytes: &[u8]) -> RecoveryObserverArtifactEvidence {
    let Some(fields) = fields(bytes) else {
        return physical_format::residue(bytes);
    };
    let Some(page_bytes) = fields.page_count.checked_mul(PAGE_ROW_BYTES) else {
        return physical_format::residue(bytes);
    };
    let Some(total) = HEADER_BYTES
        .checked_add(page_bytes)
        .and_then(|value| value.checked_add(fields.identity_bytes))
        .and_then(|value| value.checked_add(FOOTER_BYTES))
    else {
        return physical_format::residue(bytes);
    };
    if total != bytes.len()
        || fields.page_count == 0
        || fields.manifest_generation == 0
        || fields.root_reference == 0
        || fields.root_generation == 0
        || fields.covered_start >= fields.covered_end
        || fields.redo_lsn < fields.covered_start
        || fields.redo_lsn >= fields.covered_end
        || fields.durable_lsn < fields.redo_lsn
        || fields.durable_lsn > fields.covered_end
    {
        return physical_format::residue(bytes);
    }
    let footer_start = bytes.len() - FOOTER_BYTES;
    let expected: [u8; 32] = Sha256::digest(&bytes[..footer_start]).into();
    if bytes[footer_start..] != expected {
        return physical_format::residue(bytes);
    }
    let rows_end = HEADER_BYTES + page_bytes;
    let identity = &bytes[rows_end..rows_end + fields.identity_bytes];
    if identity.is_empty() || std::str::from_utf8(identity).is_err() {
        return physical_format::residue(bytes);
    }
    let mut page_digest = EvidenceDigestBuilder::new(b"worth.store.recovery-observer.page-lsn.v1");
    let mut minimum = u64::MAX;
    let mut maximum = 0_u64;
    for row in bytes[HEADER_BYTES..rows_end].chunks_exact(PAGE_ROW_BYTES) {
        let Some(segment) = physical_format::read_u64(row, 0) else {
            return physical_format::residue(bytes);
        };
        let Some(page) = physical_format::read_u64(row, 8) else {
            return physical_format::residue(bytes);
        };
        let Some(generation) = physical_format::read_u64(row, 16) else {
            return physical_format::residue(bytes);
        };
        let Some(page_lsn) = physical_format::read_u64(row, 24) else {
            return physical_format::residue(bytes);
        };
        if segment == 0 || page == 0 || generation == 0 || page_lsn < fields.redo_lsn {
            return physical_format::residue(bytes);
        }
        minimum = minimum.min(page_lsn);
        maximum = maximum.max(page_lsn);
        page_digest.record(row);
    }
    let page_digest = page_digest.finish();
    let mut generation_digest =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    generation_digest.record(&bytes[10..74]);
    let generation_digest = generation_digest.finish();
    RecoveryObserverArtifactEvidence {
        generation_links: generation_digest,
        checkpoint: Some(RecoveryObserverCheckpointObservation {
            page_count: fields.page_count as u64,
            covered_lsn: (fields.covered_start, fields.covered_end),
            redo_lsn: fields.redo_lsn,
            durable_checkpoint_lsn: fields.durable_lsn,
            digest: physical_format::digest_bytes(bytes),
        }),
        page_lsns: RecoveryObserverPageLsnObservation {
            count: page_digest.observations(),
            minimum: Some(minimum),
            maximum: Some(maximum),
            digest: page_digest.digest(),
        },
        ..RecoveryObserverArtifactEvidence::empty()
    }
}

#[derive(Debug, Clone, Copy)]
struct CheckpointFields {
    manifest_generation: u64,
    durable_lsn: u64,
    root_reference: u64,
    root_generation: u64,
    covered_start: u64,
    covered_end: u64,
    redo_lsn: u64,
    page_count: usize,
    identity_bytes: usize,
}

fn fields(bytes: &[u8]) -> Option<CheckpointFields> {
    if bytes.len() < HEADER_BYTES || physical_format::read_u16(bytes, 8)? != 1 {
        return None;
    }
    Some(CheckpointFields {
        manifest_generation: physical_format::read_u64(bytes, 10)?,
        durable_lsn: physical_format::read_u64(bytes, 18)?,
        root_reference: physical_format::read_u64(bytes, 26)?,
        root_generation: physical_format::read_u64(bytes, 34)?,
        covered_start: physical_format::read_u64(bytes, 42)?,
        covered_end: physical_format::read_u64(bytes, 50)?,
        redo_lsn: physical_format::read_u64(bytes, 58)?,
        page_count: usize::try_from(physical_format::read_u64(bytes, 66)?).ok()?,
        identity_bytes: usize::try_from(physical_format::read_u32(bytes, 74)?).ok()?,
    })
}
