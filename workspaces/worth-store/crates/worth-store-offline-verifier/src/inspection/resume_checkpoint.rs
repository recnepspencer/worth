use std::time::Duration;

use crate::OfflinePhysicalArtifactFamily;
use worth_store_physical_backend::{OfflineMediaClosureEntry, OfflineMediaFileIdentity};

use super::{OfflineInspectionCounters, OfflineWalkedFile};
use crate::inspection::resume_checkpoint_codec::{decode_checkpoint, encode_checkpoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineInspectionCheckpointCodecDenial {
    InvalidEncoding,
    AllocationFailed,
    SizeLimitExceeded,
    FileLimitExceeded,
    OwnedAllocationLimitExceeded { admitted: u64, limit: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineInspectionCheckpoint {
    pub(super) basis_identity: String,
    pub(super) file_index: usize,
    pub(super) offset: u64,
    pub(super) counters: OfflineInspectionCounters,
    pub(super) elapsed: Duration,
    pub(super) completed: Vec<CheckpointFileObservation>,
    pub(super) partial_source: Option<CheckpointSourceIdentity>,
    pub(super) partial_digest: Option<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CheckpointSourceIdentity {
    length: u64,
    metadata_fingerprint: [u8; 32],
    physical_alias_group: u64,
    physical_key_fingerprint: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CheckpointFileObservation {
    file_index: usize,
    source: CheckpointSourceIdentity,
    family: OfflinePhysicalArtifactFamily,
    content_digest: [u8; 32],
}

impl OfflineInspectionCheckpoint {
    pub fn basis_identity(&self) -> &str {
        &self.basis_identity
    }
    pub const fn file_index(&self) -> usize {
        self.file_index
    }
    pub const fn offset(&self) -> u64 {
        self.offset
    }
    pub const fn observed_bytes(&self) -> u64 {
        self.counters.bytes_read()
    }
    pub const fn counters(&self) -> OfflineInspectionCounters {
        self.counters
    }
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }
    pub fn encode(&self) -> Result<Vec<u8>, OfflineInspectionCheckpointCodecDenial> {
        encode_checkpoint(self)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, OfflineInspectionCheckpointCodecDenial> {
        decode_checkpoint(bytes, 64 * 1024 * 1024)
    }
    pub(crate) fn decode_with_owned_allocation_limit(
        bytes: &[u8],
        maximum_owned_allocation_bytes: u64,
    ) -> Result<Self, OfflineInspectionCheckpointCodecDenial> {
        decode_checkpoint(bytes, maximum_owned_allocation_bytes)
    }
}

impl CheckpointSourceIdentity {
    pub(super) fn from_source(source: &OfflineMediaFileIdentity) -> Self {
        Self {
            length: source.length(),
            metadata_fingerprint: source.metadata_fingerprint(),
            physical_alias_group: source.physical_alias_group(),
            physical_key_fingerprint: source.physical_key_fingerprint(),
        }
    }

    pub(super) fn matches(&self, source: &OfflineMediaFileIdentity) -> bool {
        self.length == source.length()
            && self.metadata_fingerprint == source.metadata_fingerprint()
            && self.physical_alias_group == source.physical_alias_group()
            && self.physical_key_fingerprint == source.physical_key_fingerprint()
    }
}

impl CheckpointFileObservation {
    pub(super) fn from_walked(file_index: usize, file: &OfflineWalkedFile) -> Self {
        Self {
            file_index,
            source: CheckpointSourceIdentity::from_source(file.source()),
            family: file.family(),
            content_digest: file.content_digest(),
        }
    }

    pub(super) fn admits(
        &self,
        file_index: usize,
        source: &OfflineMediaFileIdentity,
        family: OfflinePhysicalArtifactFamily,
        expected: &OfflineMediaClosureEntry,
    ) -> bool {
        self.file_index == file_index
            && self.source.matches(source)
            && self.family == family
            && self.content_digest == expected.content_digest()
            && source.length() == expected.bytes()
    }

    pub(super) const fn file_index(&self) -> usize {
        self.file_index
    }
    pub(super) const fn source(&self) -> &CheckpointSourceIdentity {
        &self.source
    }
    pub(super) const fn family(&self) -> OfflinePhysicalArtifactFamily {
        self.family
    }
    pub(super) const fn content_digest(&self) -> [u8; 32] {
        self.content_digest
    }
}

impl CheckpointSourceIdentity {
    pub(super) const fn from_encoded(
        length: u64,
        metadata_fingerprint: [u8; 32],
        physical_alias_group: u64,
        physical_key_fingerprint: [u8; 32],
    ) -> Self {
        Self {
            length,
            metadata_fingerprint,
            physical_alias_group,
            physical_key_fingerprint,
        }
    }
    pub(super) const fn encoded_fields(&self) -> (u64, [u8; 32], u64, [u8; 32]) {
        (
            self.length,
            self.metadata_fingerprint,
            self.physical_alias_group,
            self.physical_key_fingerprint,
        )
    }
}

impl CheckpointFileObservation {
    pub(super) const fn from_encoded(
        file_index: usize,
        source: CheckpointSourceIdentity,
        family: OfflinePhysicalArtifactFamily,
        content_digest: [u8; 32],
    ) -> Self {
        Self {
            file_index,
            source,
            family,
            content_digest,
        }
    }
}
