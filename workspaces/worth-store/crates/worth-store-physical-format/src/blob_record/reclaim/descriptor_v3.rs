use sha2::{Digest, Sha256};

use super::super::envelope::{
    canonical_frame_sha256, decode, nonzero_32, visit_canonical_frame, BLOB_RECORD_HEADER_BYTES,
};
use super::super::{BlobRecordDenial, BlobRecordKind};
use super::descriptor_v2::{BlobReclaimDescriptorV2, PAYLOAD_BYTES as V2_PAYLOAD_BYTES};
use super::{BlobReclaimSourceKind, OriginalDropReservationRequestV1};

const CUSTODY_VERSION: u8 = 1;
const PAYLOAD_BYTES: usize = V2_PAYLOAD_BYTES + 1 + 8 * 32 + 2 * 8;
const CUSTODY_DIGEST_DOMAIN: &[u8] = b"store.physical.released-drop-custody.v1";

/// Store-attested physical custody captured while the selected source bytes
/// are protected. A digest alone grants no release authority: consumers must
/// bind this to the selected descriptor, reservation, and durable drop fate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleasedDropCustodyV1 {
    source_root_frame_sha256: [u8; 32],
    source_free_space_frame_sha256: [u8; 32],
    selected_route_inventory_sha256: [u8; 32],
    authenticated_closure_edge_sha256: [u8; 32],
    external_edge_audit_sha256: [u8; 32],
    postorder_drop_sha256: [u8; 32],
    request: OriginalDropReservationRequestV1,
}

impl ReleasedDropCustodyV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_root_frame_sha256: [u8; 32],
        source_free_space_frame_sha256: [u8; 32],
        selected_route_inventory_sha256: [u8; 32],
        authenticated_closure_edge_sha256: [u8; 32],
        external_edge_audit_sha256: [u8; 32],
        postorder_drop_sha256: [u8; 32],
        request: OriginalDropReservationRequestV1,
    ) -> Result<Self, BlobRecordDenial> {
        Ok(Self {
            source_root_frame_sha256: nonzero_32(source_root_frame_sha256)?,
            source_free_space_frame_sha256: nonzero_32(source_free_space_frame_sha256)?,
            selected_route_inventory_sha256: nonzero_32(selected_route_inventory_sha256)?,
            authenticated_closure_edge_sha256: nonzero_32(authenticated_closure_edge_sha256)?,
            external_edge_audit_sha256: nonzero_32(external_edge_audit_sha256)?,
            postorder_drop_sha256: nonzero_32(postorder_drop_sha256)?,
            request,
        })
    }

    pub const fn source_root_frame_sha256(self) -> [u8; 32] {
        self.source_root_frame_sha256
    }
    pub const fn source_free_space_frame_sha256(self) -> [u8; 32] {
        self.source_free_space_frame_sha256
    }
    pub const fn selected_route_inventory_sha256(self) -> [u8; 32] {
        self.selected_route_inventory_sha256
    }
    pub const fn authenticated_closure_edge_sha256(self) -> [u8; 32] {
        self.authenticated_closure_edge_sha256
    }
    pub const fn external_edge_audit_sha256(self) -> [u8; 32] {
        self.external_edge_audit_sha256
    }
    pub const fn postorder_drop_sha256(self) -> [u8; 32] {
        self.postorder_drop_sha256
    }
    pub const fn request(self) -> OriginalDropReservationRequestV1 {
        self.request
    }
}

/// V2's exact chain prefix followed by a versioned, bounded custody transfer.
/// Only released generations may use it; failed-ingest residue keeps its V1/V2
/// descriptor semantics and cannot acquire a release certificate by decoding.
/// The drop request fingerprint is computed over the V2 base frame plus
/// placement, never this V3 frame: the latter embeds that fingerprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobReclaimDescriptorV3 {
    base: BlobReclaimDescriptorV2,
    custody: ReleasedDropCustodyV1,
}

impl BlobReclaimDescriptorV3 {
    pub const fn encoded_frame_bytes() -> usize {
        BLOB_RECORD_HEADER_BYTES + PAYLOAD_BYTES
    }

    pub fn new(
        base: BlobReclaimDescriptorV2,
        custody: ReleasedDropCustodyV1,
    ) -> Result<Self, BlobRecordDenial> {
        if base.source_kind() != BlobReclaimSourceKind::ReleasedGeneration {
            return Err(BlobRecordDenial::InvalidReclaimDescriptor);
        }
        Ok(Self { base, custody })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut frame = Vec::with_capacity(BLOB_RECORD_HEADER_BYTES + PAYLOAD_BYTES);
        visit_canonical_frame(
            BlobRecordKind::ReclaimDescriptorV3,
            PAYLOAD_BYTES,
            |emit| self.visit_payload(emit),
            &mut |part| frame.extend_from_slice(part),
        )
        .expect("fixed released-drop custody fits control-frame ceiling");
        frame
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = decode(bytes)?;
        if frame.kind != BlobRecordKind::ReclaimDescriptorV3 {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload)
    }

    pub(in crate::blob_record) fn decode_payload(payload: &[u8]) -> Result<Self, BlobRecordDenial> {
        if payload.len() != PAYLOAD_BYTES {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        if payload[V2_PAYLOAD_BYTES] != CUSTODY_VERSION {
            return Err(BlobRecordDenial::InvalidReclaimDescriptor);
        }
        let base = BlobReclaimDescriptorV2::decode_payload(&payload[..V2_PAYLOAD_BYTES])?;
        let mut cursor = V2_PAYLOAD_BYTES + 1;
        let mut next_digest = || {
            let digest = payload[cursor..cursor + 32]
                .try_into()
                .expect("fixed custody digest");
            cursor += 32;
            digest
        };
        let source = next_digest();
        let source_free = next_digest();
        let inventory = next_digest();
        let closure = next_digest();
        let external = next_digest();
        let postorder = next_digest();
        let idempotency = next_digest();
        let fingerprint = next_digest();
        let issuance = u64::from_le_bytes(payload[cursor..cursor + 8].try_into().unwrap());
        let expiry = u64::from_le_bytes(payload[cursor + 8..cursor + 16].try_into().unwrap());
        let request =
            OriginalDropReservationRequestV1::new(idempotency, fingerprint, issuance, expiry)?;
        Self::new(
            base,
            ReleasedDropCustodyV1::new(
                source,
                source_free,
                inventory,
                closure,
                external,
                postorder,
                request,
            )?,
        )
    }

    pub fn custody_digest(self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(CUSTODY_DIGEST_DOMAIN);
        self.visit_payload(&mut |part| digest.update(part));
        digest.finalize().into()
    }

    pub fn canonical_frame_sha256(self) -> [u8; 32] {
        canonical_frame_sha256(BlobRecordKind::ReclaimDescriptorV3, PAYLOAD_BYTES, |emit| {
            self.visit_payload(emit)
        })
    }

    pub const fn base(self) -> BlobReclaimDescriptorV2 {
        self.base
    }
    pub const fn custody(self) -> ReleasedDropCustodyV1 {
        self.custody
    }

    /// Noncyclic descriptor preimage for the existing drop request fingerprint.
    /// The placement component is supplied by the Store's operation owner.
    pub fn request_fingerprint_descriptor_bytes(self) -> Vec<u8> {
        self.base.encode()
    }

    fn visit_payload(self, emit: &mut dyn FnMut(&[u8])) {
        self.base.visit_payload(emit);
        emit(&[CUSTODY_VERSION]);
        for digest in [
            self.custody.source_root_frame_sha256,
            self.custody.source_free_space_frame_sha256,
            self.custody.selected_route_inventory_sha256,
            self.custody.authenticated_closure_edge_sha256,
            self.custody.external_edge_audit_sha256,
            self.custody.postorder_drop_sha256,
            self.custody.request.idempotency(),
            self.custody.request.fingerprint(),
        ] {
            emit(&digest);
        }
        emit(
            &self
                .custody
                .request
                .lease_issuance_generation()
                .to_le_bytes(),
        );
        emit(&self.custody.request.lease_expiry_generation().to_le_bytes());
    }
}

#[cfg(test)]
#[path = "descriptor_v3/tests.rs"]
mod tests;
