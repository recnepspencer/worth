use crate::PersistedRecordIdentity;

use super::super::envelope::{encode, nonzero_16, nonzero_32};
use super::super::{BlobRecordDenial, BlobRecordKind};
use super::basis::{read_record, write_record};
use super::MAXIMUM_DROP_SET_RECORDS;

const PAYLOAD_BYTES: usize = 138;

/// The durable drop intent bound to one manifest, not a second ID inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobReclaimDescriptorV1 {
    store: [u8; 16],
    reclaim_attempt: [u8; 16],
    source_basis_digest: [u8; 32],
    manifest_record: PersistedRecordIdentity,
    manifest_frame_sha256: [u8; 32],
    manifest_count: u16,
    source_root_generation: u64,
    candidate_root_generation: u64,
}

impl BlobReclaimDescriptorV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: [u8; 16],
        reclaim_attempt: [u8; 16],
        source_basis_digest: [u8; 32],
        manifest_record: PersistedRecordIdentity,
        manifest_frame_sha256: [u8; 32],
        manifest_count: u16,
        source_root_generation: u64,
        candidate_root_generation: u64,
    ) -> Result<Self, BlobRecordDenial> {
        if manifest_count == 0
            || usize::from(manifest_count) > MAXIMUM_DROP_SET_RECORDS
            || source_root_generation == 0
            || source_root_generation.checked_add(1) != Some(candidate_root_generation)
        {
            return Err(BlobRecordDenial::InvalidReclaimDescriptor);
        }
        Ok(Self {
            store: nonzero_16(store)?,
            reclaim_attempt: nonzero_16(reclaim_attempt)?,
            source_basis_digest: nonzero_32(source_basis_digest)?,
            manifest_record,
            manifest_frame_sha256: nonzero_32(manifest_frame_sha256)?,
            manifest_count,
            source_root_generation,
            candidate_root_generation,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(PAYLOAD_BYTES);
        payload.extend_from_slice(&self.store);
        payload.extend_from_slice(&self.reclaim_attempt);
        payload.extend_from_slice(&self.source_basis_digest);
        write_record(&mut payload, self.manifest_record);
        payload.extend_from_slice(&self.manifest_frame_sha256);
        payload.extend_from_slice(&self.manifest_count.to_le_bytes());
        payload.extend_from_slice(&self.source_root_generation.to_le_bytes());
        payload.extend_from_slice(&self.candidate_root_generation.to_le_bytes());
        encode(BlobRecordKind::ReclaimDescriptor, &payload)
            .expect("fixed reclaim descriptor fits control-frame ceiling")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::ReclaimDescriptor {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload)
    }

    pub(in crate::blob_record) fn decode_payload(payload: &[u8]) -> Result<Self, BlobRecordDenial> {
        if payload.len() != PAYLOAD_BYTES {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        Self::new(
            payload[..16].try_into().expect("fixed store"),
            payload[16..32].try_into().expect("fixed attempt"),
            payload[32..64].try_into().expect("fixed basis digest"),
            read_record(&payload[64..88])?,
            payload[88..120].try_into().expect("fixed manifest digest"),
            u16::from_le_bytes(payload[120..122].try_into().expect("fixed count")),
            u64::from_le_bytes(payload[122..130].try_into().expect("fixed source root")),
            u64::from_le_bytes(payload[130..138].try_into().expect("fixed candidate root")),
        )
    }

    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn reclaim_attempt(self) -> [u8; 16] {
        self.reclaim_attempt
    }
    pub const fn source_basis_digest(self) -> [u8; 32] {
        self.source_basis_digest
    }
    pub const fn manifest_record(self) -> PersistedRecordIdentity {
        self.manifest_record
    }
    pub const fn manifest_frame_sha256(self) -> [u8; 32] {
        self.manifest_frame_sha256
    }
    pub const fn manifest_count(self) -> u16 {
        self.manifest_count
    }
    pub const fn source_root_generation(self) -> u64 {
        self.source_root_generation
    }
    pub const fn candidate_root_generation(self) -> u64 {
        self.candidate_root_generation
    }
}
