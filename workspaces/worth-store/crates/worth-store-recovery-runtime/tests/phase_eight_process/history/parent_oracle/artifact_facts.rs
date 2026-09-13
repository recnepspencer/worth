use sha2::{Digest, Sha256};

use super::evidence_digest::DigestObservation;
use super::{durable, wire};

const DURABLE_MAGIC: &[u8; 8] = b"WRC5FRM\0";
const WAL_MAGIC: &[u8; 8] = b"WORTHWAL";
const CHECKPOINT_STREAM_MAGIC: &[u8; 8] = b"WCP7REC\0";

#[derive(Debug, Clone, Copy)]
pub(crate) struct ArtifactFacts {
    pub(crate) generation: bool,
    pub(crate) generation_links: DigestObservation,
    pub(crate) selector: Option<SelectorFacts>,
    pub(crate) checkpoint: Option<CheckpointFacts>,
    pub(crate) wal: Option<WalFacts>,
    pub(crate) wal_residue: Option<ResidueFacts>,
    pub(crate) page: Option<PageFacts>,
    pub(crate) manifest: Option<ManifestFacts>,
    pub(crate) residue: Option<ResidueFacts>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SelectorFacts {
    pub(crate) identity: u64,
    pub(crate) linked: Option<u64>,
    pub(crate) store: [u8; 16],
    pub(crate) role: u8,
    pub(crate) generation: u64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CheckpointFacts {
    pub(crate) sequence: u64,
    pub(crate) page_count: u64,
    pub(crate) covered: (u64, u64),
    pub(crate) redo: u64,
    pub(crate) durable: u64,
    pub(crate) generation_links: DigestObservation,
    pub(crate) digest: [u8; 32],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct WalFacts {
    pub(crate) segment: Option<u64>,
    pub(crate) generation: Option<u64>,
    pub(crate) valid_bytes: u64,
    pub(crate) observed_bytes: u64,
    pub(crate) frames: u64,
    pub(crate) first: Option<u64>,
    pub(crate) last: Option<u64>,
    pub(crate) digest: [u8; 32],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PageFacts {
    pub(crate) count: u64,
    pub(crate) minimum: Option<u64>,
    pub(crate) maximum: Option<u64>,
    pub(crate) digest: DigestObservation,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ManifestFacts {
    pub(crate) count: u64,
    pub(crate) members: u64,
    pub(crate) digest: [u8; 32],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResidueFacts {
    pub(crate) len: u64,
    pub(crate) digest: [u8; 32],
}

pub(crate) fn observe_artifact(bytes: &[u8]) -> ArtifactFacts {
    if bytes.get(..8) == Some(&WAL_MAGIC[..]) {
        return wire::observe_wal(bytes);
    }
    if bytes.get(..8) == Some(&CHECKPOINT_STREAM_MAGIC[..]) {
        if let Some(facts) = wire::observe_checkpoint(bytes) {
            return ArtifactFacts {
                generation: true,
                generation_links: facts.generation_links,
                checkpoint: Some(facts),
                ..empty_facts()
            };
        }
    }
    if bytes.get(..8) == Some(&DURABLE_MAGIC[..]) {
        return durable::observe(bytes);
    }
    ArtifactFacts {
        residue: Some(residue(bytes)),
        ..empty_facts()
    }
}

pub(crate) fn observe_artifact_at_path(path: &str, bytes: &[u8]) -> ArtifactFacts {
    if path.starts_with("staging/") && path.ends_with(".candidate") {
        return ArtifactFacts {
            residue: Some(residue(bytes)),
            ..empty_facts()
        };
    }
    observe_artifact(bytes)
}

pub(crate) const fn empty_facts() -> ArtifactFacts {
    ArtifactFacts {
        generation: false,
        generation_links: DigestObservation::empty(),
        selector: None,
        checkpoint: None,
        wal: None,
        wal_residue: None,
        page: None,
        manifest: None,
        residue: None,
    }
}

pub(crate) fn residue(bytes: &[u8]) -> ResidueFacts {
    ResidueFacts {
        len: bytes.len() as u64,
        digest: Sha256::digest(bytes).into(),
    }
}

pub(crate) fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    bytes
        .get(offset..offset + 2)?
        .try_into()
        .ok()
        .map(u16::from_le_bytes)
}

pub(crate) fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    bytes
        .get(offset..offset + 4)?
        .try_into()
        .ok()
        .map(u32::from_le_bytes)
}

pub(crate) fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    bytes
        .get(offset..offset + 8)?
        .try_into()
        .ok()
        .map(u64::from_le_bytes)
}
