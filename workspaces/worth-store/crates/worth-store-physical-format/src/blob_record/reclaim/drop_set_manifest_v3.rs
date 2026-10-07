use crate::PersistedRecordIdentity;

use super::super::envelope::{nonzero_16, visit_canonical_frame, BLOB_RECORD_HEADER_BYTES};
use super::super::{BlobRecordDenial, BlobRecordKind};
use super::drop_set_manifest::digest_dropped;
use super::{BlobReclaimSourceBasisV1, BlobReclaimSourceKind, MAXIMUM_DROP_SET_RECORDS};

#[path = "drop_set_manifest_v3/view.rs"]
mod view;
pub use view::DropSetManifestV3View;

pub(super) const NEVER_RESERVED: u8 = 1;
pub(super) const MINIMUM_PAYLOAD_BYTES: usize = 78;

/// Versioned manifest custody. Its source is a record of admission, not a
/// public constructor for semantic release permission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropSetManifestV3 {
    store: [u8; 16],
    reclaim_attempt: [u8; 16],
    source_basis: BlobReclaimSourceBasisV1,
    dropped_digest: [u8; 32],
    dropped: Box<[PersistedRecordIdentity]>,
    manifest_selected_generation: u64,
}

impl DropSetManifestV3 {
    /// Exact canonical frame length without allocating the encoded frame.
    pub fn encoded_frame_bytes(&self) -> usize {
        BLOB_RECORD_HEADER_BYTES
            + MINIMUM_PAYLOAD_BYTES
            + self.source_basis.encoded_len()
            + 24 * self.dropped.len()
    }

    pub fn new(
        store: [u8; 16],
        reclaim_attempt: [u8; 16],
        source_basis: BlobReclaimSourceBasisV1,
        dropped: Vec<PersistedRecordIdentity>,
        manifest_selected_generation: u64,
    ) -> Result<Self, BlobRecordDenial> {
        if manifest_selected_generation == 0
            || dropped.is_empty()
            || dropped.len() > MAXIMUM_DROP_SET_RECORDS
            || dropped.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        match source_basis {
            BlobReclaimSourceBasisV1::FailedIngest(basis)
                if dropped.contains(&basis.declaration_record())
                    || dropped.contains(&basis.abandoned_record()) =>
            {
                return Err(BlobRecordDenial::InvalidDropSet);
            }
            BlobReclaimSourceBasisV1::ReleasedGeneration(basis)
                if basis.publication().store() != store =>
            {
                return Err(BlobRecordDenial::InvalidDropSet);
            }
            _ => {}
        }
        Ok(Self {
            store: nonzero_16(store)?,
            reclaim_attempt: nonzero_16(reclaim_attempt)?,
            source_basis,
            dropped_digest: digest_dropped(&dropped),
            dropped: dropped.into_boxed_slice(),
            manifest_selected_generation,
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let view = DropSetManifestV3View::from_owned(self);
        let frame_bytes = self.encoded_frame_bytes();
        let payload_len = frame_bytes - BLOB_RECORD_HEADER_BYTES;
        let mut frame = Vec::with_capacity(frame_bytes);
        visit_canonical_frame(
            BlobRecordKind::DropSetManifestV3,
            payload_len,
            |emit| view.visit_payload(emit),
            &mut |part| frame.extend_from_slice(part),
        )
        .expect("bounded versioned drop set fits control-frame ceiling");
        frame
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::DropSetManifestV3 {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload)
    }

    pub(in crate::blob_record) fn decode_payload(payload: &[u8]) -> Result<Self, BlobRecordDenial> {
        let view = DropSetManifestV3View::decode_payload(payload)?;
        Self::new(
            view.store(),
            view.reclaim_attempt(),
            view.source_basis(),
            view.collect_owned(),
            view.never_reserved_slot_generation(),
        )
    }

    pub fn canonical_frame_sha256(&self) -> [u8; 32] {
        DropSetManifestV3View::from_owned(self).canonical_frame_sha256()
    }

    pub const fn store(&self) -> [u8; 16] {
        self.store
    }
    pub const fn reclaim_attempt(&self) -> [u8; 16] {
        self.reclaim_attempt
    }
    pub const fn source_basis(&self) -> BlobReclaimSourceBasisV1 {
        self.source_basis
    }
    pub const fn source_kind(&self) -> BlobReclaimSourceKind {
        self.source_basis.kind()
    }
    pub fn source_basis_digest(&self) -> [u8; 32] {
        self.source_basis.digest(self.store)
    }
    pub const fn dropped_digest(&self) -> [u8; 32] {
        self.dropped_digest
    }
    pub fn dropped(&self) -> &[PersistedRecordIdentity] {
        &self.dropped
    }
    pub fn count(&self) -> u16 {
        self.dropped.len() as u16
    }
    pub const fn never_reserved_slot_generation(&self) -> u64 {
        self.manifest_selected_generation
    }
}
