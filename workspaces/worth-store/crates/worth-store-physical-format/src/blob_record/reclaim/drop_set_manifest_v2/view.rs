use crate::PersistedRecordIdentity;

use super::super::super::envelope::canonical_frame_sha256;
use super::super::super::{BlobRecordDenial, BlobRecordKind};
use super::super::drop_set_manifest::DropSetManifestV1View;
use super::super::FailedIngestReclaimBasisV1;
use super::{NEVER_RESERVED, SLOT_BYTES};

/// Validated current failed-ingest V2 manifest grammar over borrowed frame bytes.
#[derive(Debug, Clone, Copy)]
pub struct DropSetManifestV2View<'a> {
    drop_set: DropSetManifestV1View<'a>,
    manifest_selected_generation: u64,
    payload: &'a [u8],
}

impl<'a> DropSetManifestV2View<'a> {
    pub fn decode(bytes: &'a [u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::super::super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::DropSetManifestV2 {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload)
    }

    pub(in crate::blob_record) fn decode_payload(
        payload: &'a [u8],
    ) -> Result<Self, BlobRecordDenial> {
        let split = payload
            .len()
            .checked_sub(SLOT_BYTES)
            .ok_or(BlobRecordDenial::LengthMismatch)?;
        if payload[split + 8] != NEVER_RESERVED {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        let manifest_selected_generation = u64::from_le_bytes(
            payload[split..split + 8]
                .try_into()
                .expect("slot length admitted"),
        );
        if manifest_selected_generation == 0 {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        Ok(Self {
            drop_set: DropSetManifestV1View::decode_payload(&payload[..split])?,
            manifest_selected_generation,
            payload,
        })
    }

    pub const fn store(self) -> [u8; 16] {
        self.drop_set.store()
    }
    pub const fn reclaim_attempt(self) -> [u8; 16] {
        self.drop_set.reclaim_attempt()
    }
    pub const fn source_basis(self) -> FailedIngestReclaimBasisV1 {
        self.drop_set.source_basis()
    }
    pub fn source_basis_digest(self) -> [u8; 32] {
        self.drop_set.source_basis_digest()
    }
    pub const fn dropped_digest(self) -> [u8; 32] {
        self.drop_set.dropped_digest()
    }
    pub const fn count(self) -> u16 {
        self.drop_set.count()
    }
    pub const fn never_reserved_slot_generation(self) -> u64 {
        self.manifest_selected_generation
    }
    pub(super) const fn drop_set(self) -> DropSetManifestV1View<'a> {
        self.drop_set
    }
    pub fn contains_record(self, record: PersistedRecordIdentity) -> bool {
        self.drop_set.contains_record(record)
    }
    pub fn canonical_frame_sha256(self) -> [u8; 32] {
        canonical_frame_sha256(
            BlobRecordKind::DropSetManifestV2,
            self.payload.len(),
            |emit| emit(self.payload),
        )
    }
}
