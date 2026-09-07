//! Legacy structural inspection evidence, never C.9 validator authority.
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OfflinePhysicalArtifactFamily {
    Manifest,
    Page,
    Extent,
    Wal,
    Index,
    BlobChunk,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineStructuralObservation {
    family: OfflinePhysicalArtifactFamily,
    offset: u64,
    length: u64,
    content_digest: [u8; 32],
}

impl OfflineStructuralObservation {
    pub const fn family(&self) -> OfflinePhysicalArtifactFamily {
        self.family
    }
    pub const fn offset(&self) -> u64 {
        self.offset
    }
    pub const fn length(&self) -> u64 {
        self.length
    }
    pub const fn content_digest(&self) -> [u8; 32] {
        self.content_digest
    }
}

pub fn observe_bounded_physical_bytes(
    family: OfflinePhysicalArtifactFamily,
    offset: u64,
    bytes: &[u8],
) -> OfflineStructuralObservation {
    OfflineStructuralObservation {
        family,
        offset,
        length: bytes.len() as u64,
        content_digest: Sha256::digest(bytes).into(),
    }
}

pub fn classify_offline_artifact_family(name: &str) -> OfflinePhysicalArtifactFamily {
    let lower = name.to_ascii_lowercase();
    if lower.contains("manifest") {
        OfflinePhysicalArtifactFamily::Manifest
    } else if lower.ends_with(".page") || lower.contains("pages") {
        OfflinePhysicalArtifactFamily::Page
    } else if lower.ends_with(".extent") || lower.contains("extents") {
        OfflinePhysicalArtifactFamily::Extent
    } else if lower.ends_with(".wal") || lower.contains("wal") {
        OfflinePhysicalArtifactFamily::Wal
    } else if lower.ends_with(".index") || lower.contains("index") {
        OfflinePhysicalArtifactFamily::Index
    } else if lower.ends_with(".chunk") || lower.contains("blob") {
        OfflinePhysicalArtifactFamily::BlobChunk
    } else {
        OfflinePhysicalArtifactFamily::Unknown
    }
}
