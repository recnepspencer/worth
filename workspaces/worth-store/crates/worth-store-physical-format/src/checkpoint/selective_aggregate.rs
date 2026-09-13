use sha2::{Digest, Sha256};

use super::CheckpointStreamDecodeDenial;

/// Canonical ordered aggregate used by checkpoint dirty-basis and binding records.
#[derive(Debug, Clone)]
pub struct CheckpointSelectiveRecordAggregate {
    record_count: u64,
    encoded_bytes: u64,
    digest: Sha256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointSelectiveRecordSummary {
    record_count: u64,
    encoded_bytes: u64,
    digest: [u8; 32],
}

impl CheckpointSelectiveRecordAggregate {
    pub fn new() -> Self {
        Self {
            record_count: 0,
            encoded_bytes: 0,
            digest: Sha256::new(),
        }
    }

    pub fn include(&mut self, record: &[u8]) -> Result<(), CheckpointStreamDecodeDenial> {
        self.record_count = self
            .record_count
            .checked_add(1)
            .ok_or(CheckpointStreamDecodeDenial::RecordCountMismatch)?;
        self.encoded_bytes = self
            .encoded_bytes
            .checked_add(record.len() as u64)
            .ok_or(CheckpointStreamDecodeDenial::RecordByteCountMismatch)?;
        self.digest.update(record);
        Ok(())
    }

    pub fn summary(&self) -> CheckpointSelectiveRecordSummary {
        CheckpointSelectiveRecordSummary {
            record_count: self.record_count,
            encoded_bytes: self.encoded_bytes,
            digest: self.digest.clone().finalize().into(),
        }
    }
}

impl Default for CheckpointSelectiveRecordAggregate {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckpointSelectiveRecordSummary {
    pub const fn record_count(self) -> u64 {
        self.record_count
    }

    pub const fn encoded_bytes(self) -> u64 {
        self.encoded_bytes
    }

    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}
