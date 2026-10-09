use crate::PersistedRecordIdentity;

use super::super::{envelope::encode, BlobRecordDenial, BlobRecordKind};
use super::{DropSetManifestV1, FailedIngestReclaimBasisV1};

#[path = "drop_set_manifest_v2/view.rs"]
mod view;
pub use view::DropSetManifestV2View;

pub(super) const SLOT_BYTES: usize = 9;
pub(super) const NEVER_RESERVED: u8 = 1;

/// Selected manifest custody that can prove the original drop was not yet
/// reserved, provided no matching Reserved transition remains selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropSetManifestV2 {
    drop_set: DropSetManifestV1,
    manifest_selected_generation: u64,
}

impl DropSetManifestV2 {
    pub fn new(
        store: [u8; 16],
        reclaim_attempt: [u8; 16],
        source_basis: FailedIngestReclaimBasisV1,
        dropped: Vec<PersistedRecordIdentity>,
        manifest_selected_generation: u64,
    ) -> Result<Self, BlobRecordDenial> {
        if manifest_selected_generation == 0 {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        Ok(Self {
            drop_set: DropSetManifestV1::new(store, reclaim_attempt, source_basis, dropped)?,
            manifest_selected_generation,
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut payload = self.drop_set.encode_payload();
        payload.extend_from_slice(&self.manifest_selected_generation.to_le_bytes());
        payload.push(NEVER_RESERVED);
        encode(BlobRecordKind::DropSetManifestV2, &payload)
            .expect("bounded versioned drop set fits control-frame ceiling")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::DropSetManifestV2 {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload)
    }

    pub(in crate::blob_record) fn decode_payload(payload: &[u8]) -> Result<Self, BlobRecordDenial> {
        let view = DropSetManifestV2View::decode_payload(payload)?;
        Ok(Self {
            drop_set: DropSetManifestV1::from_view(view.drop_set()),
            manifest_selected_generation: view.never_reserved_slot_generation(),
        })
    }

    pub const fn store(&self) -> [u8; 16] {
        self.drop_set.store()
    }
    pub const fn reclaim_attempt(&self) -> [u8; 16] {
        self.drop_set.reclaim_attempt()
    }
    pub const fn source_basis(&self) -> FailedIngestReclaimBasisV1 {
        self.drop_set.source_basis()
    }
    pub fn source_basis_digest(&self) -> [u8; 32] {
        self.drop_set.source_basis_digest()
    }
    pub fn dropped(&self) -> &[PersistedRecordIdentity] {
        self.drop_set.dropped()
    }
    pub fn count(&self) -> u16 {
        self.drop_set.count()
    }
    pub const fn drop_set(&self) -> &DropSetManifestV1 {
        &self.drop_set
    }
    pub const fn never_reserved_slot_generation(&self) -> u64 {
        self.manifest_selected_generation
    }
}
