use worth_store_blob_chunks::{
    AdmittedBlobChunkSecurity, BlobChunkSecurityScope, BlobChunkSecurityScopeDenial, BlobChunkSize,
    BlobChunkingRuleAdmission,
};
use worth_store_security::StoreAdmittedSecurityScope;

use crate::physical_runtime::PhysicalMutationDeadline;

use super::BlobObjectId;

const MIN_CHUNK_BYTES: u64 = 64 * 1024;
const MAX_CHUNK_BYTES: u64 = 256 * 1024;
const MAX_CHECKPOINT_HORIZON: u64 = u16::MAX as u64;

/// Store-admitted blob custody scope. Neither a raw fingerprint nor an
/// identity-provider claim can construct this value.
#[derive(Debug, PartialEq, Eq)]
pub struct AdmittedBlobScope(BlobChunkSecurityScope);

impl AdmittedBlobScope {
    pub fn from_store_security(
        admitted: StoreAdmittedSecurityScope,
    ) -> Result<Self, BlobChunkSecurityScopeDenial> {
        let handoff = AdmittedBlobChunkSecurity::from_admitted_security_scope(admitted)?;
        Ok(Self(BlobChunkSecurityScope::from_admitted_blob_security(
            handoff,
        )))
    }

    pub(super) fn fingerprint(&self) -> [u8; 32] {
        self.0.identity().stable_fingerprint()
    }
}

/// Relative checkpoint horizon. Store adds this to selected durable
/// checkpoint truth at declaration time; callers cannot declare an absolute
/// checkpoint sequence or a wall-clock expiry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobCheckpointLimit(u64);

impl BlobCheckpointLimit {
    pub const fn bounded_horizon(horizon: u64) -> Option<Self> {
        if horizon == 0 || horizon > MAX_CHECKPOINT_HORIZON {
            None
        } else {
            Some(Self(horizon))
        }
    }

    pub const fn horizon(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobDeclarationDenial {
    EmptyObject,
    ChunkSizeOutOfRange,
    UnsupportedChunkingRule,
}

pub struct BlobIngestDeclaration {
    object: BlobObjectId,
    rule: BlobChunkingRuleAdmission,
    total_bytes: u64,
    scope_fingerprint: [u8; 32],
    checkpoint_limit: BlobCheckpointLimit,
    deadline: PhysicalMutationDeadline,
}

impl BlobIngestDeclaration {
    pub fn new(
        object: BlobObjectId,
        chunk_size: BlobChunkSize,
        total_bytes: u64,
        scope: &AdmittedBlobScope,
        checkpoint_limit: BlobCheckpointLimit,
        deadline: PhysicalMutationDeadline,
    ) -> Result<Self, BlobDeclarationDenial> {
        if total_bytes == 0 {
            return Err(BlobDeclarationDenial::EmptyObject);
        }
        if !(MIN_CHUNK_BYTES..=MAX_CHUNK_BYTES).contains(&chunk_size.bytes()) {
            return Err(BlobDeclarationDenial::ChunkSizeOutOfRange);
        }
        let rule = BlobChunkingRuleAdmission::fixed_size(chunk_size)
            .map_err(|_| BlobDeclarationDenial::UnsupportedChunkingRule)?;
        Ok(Self {
            object,
            rule,
            total_bytes,
            scope_fingerprint: scope.fingerprint(),
            checkpoint_limit,
            deadline,
        })
    }

    pub const fn object(&self) -> BlobObjectId {
        self.object
    }

    pub const fn chunk_size(&self) -> BlobChunkSize {
        self.rule.chunk_size()
    }

    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    pub const fn checkpoint_limit(&self) -> BlobCheckpointLimit {
        self.checkpoint_limit
    }

    pub const fn deadline(&self) -> PhysicalMutationDeadline {
        self.deadline
    }

    pub(super) fn scope_fingerprint(&self) -> [u8; 32] {
        self.scope_fingerprint
    }
}
