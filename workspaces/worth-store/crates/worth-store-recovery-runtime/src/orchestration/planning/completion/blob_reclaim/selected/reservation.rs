use worth_store_physical_format::{OriginalDropReservedV1, PersistedRecordIdentity};

/// Exact selected transition required before a V2 descriptor may remove data.
#[derive(Clone, Copy)]
pub(crate) struct ExpectedReserved {
    manifest_record: PersistedRecordIdentity,
    manifest_sha256: [u8; 32],
    store: [u8; 16],
    attempt: [u8; 16],
    basis_digest: [u8; 32],
    manifest_generation: u64,
    reserved_generation: u64,
    idempotency: [u8; 32],
    fingerprint: [u8; 32],
    lease_issuance: u64,
    lease_expiry: u64,
}

impl ExpectedReserved {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        manifest_record: PersistedRecordIdentity,
        manifest_sha256: [u8; 32],
        store: [u8; 16],
        attempt: [u8; 16],
        basis_digest: [u8; 32],
        manifest_generation: u64,
        reserved_generation: u64,
        idempotency: [u8; 32],
        fingerprint: [u8; 32],
        lease_issuance: u64,
        lease_expiry: u64,
    ) -> Self {
        Self {
            manifest_record,
            manifest_sha256,
            store,
            attempt,
            basis_digest,
            manifest_generation,
            reserved_generation,
            idempotency,
            fingerprint,
            lease_issuance,
            lease_expiry,
        }
    }

    pub(super) const fn manifest_record(self) -> PersistedRecordIdentity {
        self.manifest_record
    }

    pub(super) fn matches(self, value: OriginalDropReservedV1) -> bool {
        value.store() == self.store
            && value.reclaim_attempt() == self.attempt
            && value.manifest_record() == self.manifest_record
            && value.manifest_frame_sha256() == self.manifest_sha256
            && value.source_basis_digest() == self.basis_digest
            && value.manifest_selected_generation() == self.manifest_generation
            && value.reserved_selected_generation() == self.reserved_generation
            && value.request().idempotency() == self.idempotency
            && value.request().fingerprint() == self.fingerprint
            && value.request().lease_issuance_generation() == self.lease_issuance
            && value.request().lease_expiry_generation() == self.lease_expiry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::OriginalDropReservationRequestV1;

    #[test]
    fn exact_descriptor_source_generation_is_required_after_interleaved_manifest_roots() {
        let manifest = PersistedRecordIdentity::new([1; 16], 2).unwrap();
        let request = OriginalDropReservationRequestV1::new([6; 32], [7; 32], 0, 4).unwrap();
        let reserved = OriginalDropReservedV1::new(
            [2; 16], [3; 16], manifest, [4; 32], [5; 32], 7, 9, request,
        )
        .unwrap();
        let expected = |source_generation| {
            ExpectedReserved::new(
                manifest,
                [4; 32],
                [2; 16],
                [3; 16],
                [5; 32],
                7,
                source_generation,
                [6; 32],
                [7; 32],
                0,
                4,
            )
        };
        assert!(expected(9).matches(reserved));
        assert!(!expected(8).matches(reserved));
        assert!(!expected(10).matches(reserved));
    }
}
