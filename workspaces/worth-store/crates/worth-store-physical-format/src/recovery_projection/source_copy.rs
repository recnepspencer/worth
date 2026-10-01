use super::{
    PersistedPhysicalRecoveryOperation, PersistedPhysicalRecoveryPayload,
    PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState,
};
use crate::{CurrentPhysicalRecordPlacement, PhysicalExtentCopyIntent, PhysicalExtentCopyRecord};
use sha2::{Digest, Sha256};

/// Exact reference to an independently durable source-copy intent. The intent
/// envelope LSN belongs to data frames, not to the later root publication WAL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistedExtentCopyRecipe {
    intent: PhysicalExtentCopyIntent,
    intent_lsn: u64,
    intent_digest: [u8; 32],
}

impl PersistedExtentCopyRecipe {
    pub fn new(
        intent: PhysicalExtentCopyIntent,
        intent_lsn: u64,
        intent_digest: [u8; 32],
    ) -> Option<Self> {
        let expected: [u8; 32] =
            Sha256::digest(PhysicalExtentCopyRecord::Intent(intent).encode()).into();
        (intent_lsn != 0 && intent_digest == expected).then_some(Self {
            intent,
            intent_lsn,
            intent_digest,
        })
    }
    pub const fn intent(self) -> PhysicalExtentCopyIntent {
        self.intent
    }
    pub const fn intent_lsn(self) -> u64 {
        self.intent_lsn
    }
    pub const fn intent_digest(self) -> [u8; 32] {
        self.intent_digest
    }
}

impl PersistedPhysicalRecoveryProjection {
    pub fn from_source_copy(
        source_root_generation: u64,
        root_state: PersistedPhysicalRecoveryRootState,
        recipe: PersistedExtentCopyRecipe,
    ) -> Option<Self> {
        (source_root_generation >= recipe.intent().source_root()).then_some(Self {
            source_root_generation,
            root_state,
            record_identities: vec![recipe.intent().source().record()].into_boxed_slice(),
            payload: PersistedPhysicalRecoveryPayload::SourceCopy(recipe),
            operation: PersistedPhysicalRecoveryOperation::None,
            placements: vec![CurrentPhysicalRecordPlacement::Extent(
                recipe.intent().destination(),
            )]
            .into_boxed_slice(),
            segment_updates: Box::new([]),
            manifests: Box::new([]),
        })
    }
}
