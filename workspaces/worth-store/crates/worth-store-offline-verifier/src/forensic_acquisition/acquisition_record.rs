use std::io::Write;
use std::path::Path;
use worth_foundational::CanonicalDigestId;

use sha2::{Digest, Sha256};

use super::acquisition_session::sync_directory;
use super::ForensicAcquisitionDenial;

const MAGIC: &[u8; 8] = b"WFORREC2";
const RECORD_BYTES: usize = 8 + 32 + 8 + 8 + 8 + 32 + 32 + 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DurableForensicSourceRecord {
    pub(crate) plan_identity: CanonicalDigestId,
    pub(crate) source_index: u64,
    pub(crate) source_length: u64,
    pub(crate) acquired_prefix_bytes: u64,
    pub(crate) acquired_digest: CanonicalDigestId,
    pub(crate) source_fingerprint: CanonicalDigestId,
}

impl DurableForensicSourceRecord {
    pub(crate) const fn unreadable_bytes(self) -> u64 {
        self.source_length - self.acquired_prefix_bytes
    }
}

pub(crate) fn persist(
    root: &Path,
    record: DurableForensicSourceRecord,
) -> Result<(), ForensicAcquisitionDenial> {
    let final_path = record_path(root, record.source_index);
    let pending_path = final_path.with_extension("record.pending");
    let bytes = encode(record);
    if final_path.exists() {
        let existing = std::fs::read(&final_path)?;
        if existing == bytes {
            return Ok(());
        }
        return Err(ForensicAcquisitionDenial::TargetAlreadyContainsConflict);
    }
    if pending_path.exists() {
        std::fs::remove_file(&pending_path)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&pending_path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    std::fs::rename(&pending_path, &final_path)?;
    sync_directory(root)?;
    Ok(())
}

pub(crate) fn read_all(
    root: &Path,
    source_count: usize,
) -> Result<Vec<DurableForensicSourceRecord>, ForensicAcquisitionDenial> {
    let mut records = Vec::new();
    records
        .try_reserve_exact(source_count)
        .map_err(|_| ForensicAcquisitionDenial::InvalidBufferBudget)?;
    for index in 0..source_count {
        let path = record_path(root, index as u64);
        if !path.exists() {
            break;
        }
        let bytes = std::fs::read(path)?;
        records.push(decode(&bytes)?);
    }
    Ok(records)
}

fn encode(record: DurableForensicSourceRecord) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(RECORD_BYTES);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(record.plan_identity.bytes());
    bytes.extend_from_slice(&record.source_index.to_be_bytes());
    bytes.extend_from_slice(&record.source_length.to_be_bytes());
    bytes.extend_from_slice(&record.acquired_prefix_bytes.to_be_bytes());
    bytes.extend_from_slice(record.acquired_digest.bytes());
    bytes.extend_from_slice(record.source_fingerprint.bytes());
    let checksum: [u8; 32] = Sha256::digest(&bytes).into();
    bytes.extend_from_slice(&checksum);
    bytes
}

fn decode(bytes: &[u8]) -> Result<DurableForensicSourceRecord, ForensicAcquisitionDenial> {
    if bytes.len() != RECORD_BYTES || &bytes[..8] != MAGIC {
        return Err(ForensicAcquisitionDenial::DamagedAcquisitionJournal);
    }
    let expected: [u8; 32] = Sha256::digest(&bytes[..RECORD_BYTES - 32]).into();
    if bytes[RECORD_BYTES - 32..] != expected {
        return Err(ForensicAcquisitionDenial::DamagedAcquisitionJournal);
    }
    let record = DurableForensicSourceRecord {
        plan_identity: CanonicalDigestId::new(bytes[8..40].try_into().unwrap()),
        source_index: u64::from_be_bytes(bytes[40..48].try_into().unwrap()),
        source_length: u64::from_be_bytes(bytes[48..56].try_into().unwrap()),
        acquired_prefix_bytes: u64::from_be_bytes(bytes[56..64].try_into().unwrap()),
        acquired_digest: CanonicalDigestId::new(bytes[64..96].try_into().unwrap()),
        source_fingerprint: CanonicalDigestId::new(bytes[96..128].try_into().unwrap()),
    };
    if record.acquired_prefix_bytes > record.source_length {
        return Err(ForensicAcquisitionDenial::DamagedAcquisitionJournal);
    }
    Ok(record)
}

fn record_path(root: &Path, source_index: u64) -> std::path::PathBuf {
    root.join(format!("source-{source_index:08}.record"))
}
