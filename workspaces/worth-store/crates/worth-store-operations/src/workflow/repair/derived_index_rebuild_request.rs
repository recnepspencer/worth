use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// Operator intent for a Store-owned derived-index rebuild. Paths and digests
/// classify a proposed target; they never grant file-replacement authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedIndexRepairRequest {
    operation_identity: [u8; 32],
    target: PathBuf,
    expected_target_digest: [u8; 32],
    replacement: PathBuf,
    replacement_digest: [u8; 32],
    expected_generation: u64,
    replacement_generation: u64,
    maximum_bytes: u64,
}

impl DerivedIndexRepairRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_identity: [u8; 32],
        target: impl Into<PathBuf>,
        expected_target_digest: [u8; 32],
        replacement: impl Into<PathBuf>,
        replacement_digest: [u8; 32],
        expected_generation: u64,
        replacement_generation: u64,
        maximum_bytes: u64,
    ) -> Self {
        Self {
            operation_identity,
            target: target.into(),
            expected_target_digest,
            replacement: replacement.into(),
            replacement_digest,
            expected_generation,
            replacement_generation,
            maximum_bytes,
        }
    }

    pub fn target(&self) -> &Path {
        &self.target
    }

    pub const fn expected_target_digest(&self) -> [u8; 32] {
        self.expected_target_digest
    }

    pub(super) fn lower(self) -> Result<DerivedIndexRepairPlan, DerivedIndexRepairPlanDenial> {
        if self.operation_identity == [0; 32] || self.maximum_bytes == 0 {
            return Err(DerivedIndexRepairPlanDenial::InvalidIdentity);
        }
        if self.replacement_generation <= self.expected_generation {
            return Err(DerivedIndexRepairPlanDenial::InvalidGenerationAdvance);
        }
        if self.target == self.replacement {
            return Err(DerivedIndexRepairPlanDenial::SourceTargetAlias);
        }
        let mut digest = Sha256::new();
        digest.update(b"worth-store-derived-index-rebuild-intent-v1");
        digest.update(self.operation_identity);
        digest.update(self.target.as_os_str().as_encoded_bytes());
        digest.update(self.expected_target_digest);
        digest.update(self.replacement.as_os_str().as_encoded_bytes());
        digest.update(self.replacement_digest);
        digest.update(self.expected_generation.to_be_bytes());
        digest.update(self.replacement_generation.to_be_bytes());
        digest.update(self.maximum_bytes.to_be_bytes());
        Ok(DerivedIndexRepairPlan {
            fingerprint: digest.finalize().into(),
            request: self,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedIndexRepairPlanDenial {
    InvalidIdentity,
    InvalidGenerationAdvance,
    SourceTargetAlias,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DerivedIndexRepairPlan {
    fingerprint: [u8; 32],
    request: DerivedIndexRepairRequest,
}

impl DerivedIndexRepairPlan {
    pub const fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }

    pub fn target(&self) -> &Path {
        self.request.target()
    }
}
