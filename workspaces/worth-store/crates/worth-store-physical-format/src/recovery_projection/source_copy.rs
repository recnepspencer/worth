use super::{
    decode_storage, PersistedPhysicalRecoveryOperation, PersistedPhysicalRecoveryPayload,
    PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState,
    PhysicalRecoveryDecodeFailure, PhysicalRecoveryDecodeStorage, PhysicalRecoveryProjectionDenial,
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
        Self::new_with_storage(
            intent,
            intent_lsn,
            intent_digest,
            &mut decode_storage::UnrestrictedDecodeStorage,
        )
        .ok()
    }
    pub(super) fn new_with_storage<S: PhysicalRecoveryDecodeStorage>(
        intent: PhysicalExtentCopyIntent,
        intent_lsn: u64,
        intent_digest: [u8; 32],
        storage: &mut S,
    ) -> Result<Self, PhysicalRecoveryDecodeFailure<S::Denial>> {
        if intent_lsn == 0 {
            return Err(PhysicalRecoveryProjectionDenial::Malformed.into());
        }
        let record = PhysicalExtentCopyRecord::Intent(intent);
        let encoded = record
            .encode_in_reserved(decode_storage::reserve_vec(
                record.encoded_bytes(),
                storage,
            )?)
            .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
        let expected: [u8; 32] = Sha256::digest(&encoded).into();
        (intent_digest == expected)
            .then_some(Self {
                intent,
                intent_lsn,
                intent_digest,
            })
            .ok_or_else(|| PhysicalRecoveryProjectionDenial::Malformed.into())
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
        Self::from_source_copy_with_storage(
            source_root_generation,
            root_state,
            recipe,
            &mut decode_storage::UnrestrictedDecodeStorage,
        )
        .ok()
    }
    pub(super) fn from_source_copy_with_storage<S: PhysicalRecoveryDecodeStorage>(
        source_root_generation: u64,
        root_state: PersistedPhysicalRecoveryRootState,
        recipe: PersistedExtentCopyRecipe,
        storage: &mut S,
    ) -> Result<Self, PhysicalRecoveryDecodeFailure<S::Denial>> {
        if source_root_generation < recipe.intent().source_root() {
            return Err(PhysicalRecoveryProjectionDenial::Malformed.into());
        }
        let mut records = decode_storage::reserve_vec(1, storage)?;
        records.push(recipe.intent().source().record());
        let mut placements = decode_storage::reserve_vec(1, storage)?;
        placements.push(CurrentPhysicalRecordPlacement::Extent(
            recipe.intent().destination(),
        ));
        Ok(Self {
            source_root_generation,
            root_state,
            record_identities: records.into_boxed_slice(),
            payload: PersistedPhysicalRecoveryPayload::SourceCopy(recipe),
            operation: PersistedPhysicalRecoveryOperation::None,
            placements: placements.into_boxed_slice(),
            segment_updates: Box::new([]),
            manifests: Box::new([]),
        })
    }
}
