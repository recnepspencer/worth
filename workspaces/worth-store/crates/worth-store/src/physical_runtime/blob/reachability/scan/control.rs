use worth_store_physical_format::{
    OriginalDropReservationRequestV1, PersistedRecordIdentity, ReleasedGenerationReclaimBasisV1,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::blob::reachability) enum ControlFact {
    ReleasedManifest {
        attempt: [u8; 16],
        basis: ReleasedGenerationReclaimBasisV1,
        frame_digest: [u8; 32],
        source_basis_digest: [u8; 32],
    },
    ReleasedDescriptor {
        store: [u8; 16],
        attempt: [u8; 16],
        manifest: PersistedRecordIdentity,
        manifest_digest: [u8; 32],
        source_basis_digest: [u8; 32],
        request: OriginalDropReservationRequestV1,
    },
    Reservation {
        store: [u8; 16],
        attempt: [u8; 16],
        manifest: PersistedRecordIdentity,
        manifest_digest: [u8; 32],
        source_basis_digest: [u8; 32],
        request: OriginalDropReservationRequestV1,
    },
}
