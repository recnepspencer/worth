use crate::PersistedRecordIdentity;

use super::super::{
    envelope::{canonical_frame_sha256, visit_canonical_frame, BLOB_RECORD_HEADER_BYTES},
    BlobRecordDenial, BlobRecordKind,
};
use super::basis::read_record;

const PAYLOAD_BYTES: usize = 216;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OriginalDropReservationRequestV1 {
    idempotency: [u8; 32],
    fingerprint: [u8; 32],
    lease_issuance_generation: u64,
    lease_expiry_generation: u64,
}

impl OriginalDropReservationRequestV1 {
    pub fn new(
        idempotency: [u8; 32],
        fingerprint: [u8; 32],
        lease_issuance_generation: u64,
        lease_expiry_generation: u64,
    ) -> Result<Self, BlobRecordDenial> {
        if idempotency == [0; 32]
            || fingerprint == [0; 32]
            || lease_expiry_generation <= lease_issuance_generation
        {
            return Err(BlobRecordDenial::InvalidReclaimDescriptor);
        }
        Ok(Self {
            idempotency,
            fingerprint,
            lease_issuance_generation,
            lease_expiry_generation,
        })
    }

    pub const fn idempotency(self) -> [u8; 32] {
        self.idempotency
    }
    pub const fn fingerprint(self) -> [u8; 32] {
        self.fingerprint
    }
    pub const fn lease_issuance_generation(self) -> u64 {
        self.lease_issuance_generation
    }
    pub const fn lease_expiry_generation(self) -> u64 {
        self.lease_expiry_generation
    }
}

/// Monotonic selected-root transition that closes a manifest's initial
/// NeverReserved slot before stage-2 descriptor preparation can begin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OriginalDropReservedV1 {
    store: [u8; 16],
    reclaim_attempt: [u8; 16],
    manifest_record: PersistedRecordIdentity,
    manifest_frame_sha256: [u8; 32],
    source_basis_digest: [u8; 32],
    manifest_selected_generation: u64,
    reserved_selected_generation: u64,
    request: OriginalDropReservationRequestV1,
}

impl OriginalDropReservedV1 {
    pub const fn encoded_frame_bytes() -> usize {
        BLOB_RECORD_HEADER_BYTES + PAYLOAD_BYTES
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: [u8; 16],
        reclaim_attempt: [u8; 16],
        manifest_record: PersistedRecordIdentity,
        manifest_frame_sha256: [u8; 32],
        source_basis_digest: [u8; 32],
        manifest_selected_generation: u64,
        reserved_selected_generation: u64,
        request: OriginalDropReservationRequestV1,
    ) -> Result<Self, BlobRecordDenial> {
        if store == [0; 16]
            || reclaim_attempt == [0; 16]
            || manifest_frame_sha256 == [0; 32]
            || source_basis_digest == [0; 32]
            || manifest_selected_generation == 0
            || reserved_selected_generation <= manifest_selected_generation
        {
            return Err(BlobRecordDenial::InvalidDropSet);
        }
        Ok(Self {
            store,
            reclaim_attempt,
            manifest_record,
            manifest_frame_sha256,
            source_basis_digest,
            manifest_selected_generation,
            reserved_selected_generation,
            request,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut frame = Vec::with_capacity(BLOB_RECORD_HEADER_BYTES + PAYLOAD_BYTES);
        visit_canonical_frame(
            BlobRecordKind::OriginalDropReserved,
            PAYLOAD_BYTES,
            |emit| self.visit_payload(emit),
            &mut |part| frame.extend_from_slice(part),
        )
        .expect("bounded reservation fits control-frame ceiling");
        frame
    }

    pub fn canonical_frame_sha256(self) -> [u8; 32] {
        canonical_frame_sha256(
            BlobRecordKind::OriginalDropReserved,
            PAYLOAD_BYTES,
            |emit| self.visit_payload(emit),
        )
    }

    fn visit_payload(self, emit: &mut dyn FnMut(&[u8])) {
        emit(&self.store);
        emit(&self.reclaim_attempt);
        emit(&self.manifest_record.allocation_epoch());
        emit(&self.manifest_record.ordinal().to_le_bytes());
        emit(&self.manifest_frame_sha256);
        emit(&self.source_basis_digest);
        emit(&self.manifest_selected_generation.to_le_bytes());
        emit(&self.reserved_selected_generation.to_le_bytes());
        emit(&self.request.idempotency);
        emit(&self.request.fingerprint);
        emit(&self.request.lease_issuance_generation.to_le_bytes());
        emit(&self.request.lease_expiry_generation.to_le_bytes());
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::OriginalDropReserved {
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
            read_record(&payload[32..56])?,
            payload[56..88].try_into().expect("fixed manifest digest"),
            payload[88..120].try_into().expect("fixed basis digest"),
            u64::from_le_bytes(payload[120..128].try_into().expect("fixed source")),
            u64::from_le_bytes(payload[128..136].try_into().expect("fixed candidate")),
            OriginalDropReservationRequestV1::new(
                payload[136..168].try_into().expect("fixed idempotency"),
                payload[168..200].try_into().expect("fixed fingerprint"),
                u64::from_le_bytes(payload[200..208].try_into().expect("fixed lease issuance")),
                u64::from_le_bytes(payload[208..216].try_into().expect("fixed lease expiry")),
            )?,
        )
    }

    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn reclaim_attempt(self) -> [u8; 16] {
        self.reclaim_attempt
    }
    pub const fn manifest_record(self) -> PersistedRecordIdentity {
        self.manifest_record
    }
    pub const fn manifest_frame_sha256(self) -> [u8; 32] {
        self.manifest_frame_sha256
    }
    pub const fn source_basis_digest(self) -> [u8; 32] {
        self.source_basis_digest
    }
    pub const fn manifest_selected_generation(self) -> u64 {
        self.manifest_selected_generation
    }
    pub const fn reserved_selected_generation(self) -> u64 {
        self.reserved_selected_generation
    }
    pub const fn request(self) -> OriginalDropReservationRequestV1 {
        self.request
    }
}
