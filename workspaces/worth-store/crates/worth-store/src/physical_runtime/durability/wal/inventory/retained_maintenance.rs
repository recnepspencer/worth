use sha2::{Digest, Sha256};
use worth_store_physical_backend::ArtifactTreeFile;

/// Issued only by the verified WAL inventory. Coordinates name the original
/// retained frame; a later barrier may certify it without appending a duplicate.
#[derive(Clone)]
pub(in crate::physical_runtime::durability::wal) struct RetainedMaintenanceIntent {
    artifact: ArtifactTreeFile,
    record: crate::physical_runtime::durability::retention::RetirementRecord,
    digest: [u8; 32],
    interval: (u64, u64, u64, u64, u64, u64),
}

impl RetainedMaintenanceIntent {
    pub(super) fn from_verified(
        artifact: ArtifactTreeFile,
        segment: u64,
        generation: u64,
        offset: u64,
        frame: worth_store_wal::VerifiedWalFramePayload<'_>,
    ) -> Option<Self> {
        let record =
            crate::physical_runtime::durability::retention::decode_retirement(frame.payload())?;
        if record.completion || record.release.is_none() {
            return None;
        }
        Some(Self {
            artifact,
            record,
            digest: Sha256::digest(frame.payload()).into(),
            interval: (
                segment,
                generation,
                frame.lsn_range().start().get(),
                frame.lsn_range().end_exclusive().get(),
                offset,
                frame.encoded_bytes(),
            ),
        })
    }
    pub(in crate::physical_runtime::durability::wal) fn artifact(&self) -> &ArtifactTreeFile {
        &self.artifact
    }
    pub(in crate::physical_runtime::durability::wal) const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub(in crate::physical_runtime::durability::wal) const fn record(
        &self,
    ) -> crate::physical_runtime::durability::retention::RetirementRecord {
        self.record
    }
    pub(in crate::physical_runtime::durability::wal) const fn interval(
        &self,
    ) -> (u64, u64, u64, u64, u64, u64) {
        self.interval
    }
}
