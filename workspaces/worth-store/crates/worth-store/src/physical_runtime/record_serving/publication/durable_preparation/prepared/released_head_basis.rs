use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobReclaimDescriptorV3, PersistedRecordIdentity, ReleaseCustodyHeadDenial,
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1, ReleasedGenerationReclaimBasisV1,
};

use super::PreparedPhysicalMutation;

/// Fenced Store facts that must survive preparation until the descriptor has
/// its reserved physical identity and the head transition can be fixed for WAL.
#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct PreparedReleaseHeadBasis {
    key: ReleaseCustodyHeadKeyV1,
    source_basis: ReleasedGenerationReclaimBasisV1,
    prior: Option<ReleaseCustodyHeadEntryV1>,
    descriptor: BlobReclaimDescriptorV3,
    reservation_record: PersistedRecordIdentity,
    reservation_frame_sha256: [u8; 32],
}

impl PreparedReleaseHeadBasis {
    pub(in crate::physical_runtime) fn new(
        key: ReleaseCustodyHeadKeyV1,
        source_basis: ReleasedGenerationReclaimBasisV1,
        prior: Option<ReleaseCustodyHeadEntryV1>,
        descriptor: BlobReclaimDescriptorV3,
        reservation_record: PersistedRecordIdentity,
        reservation_frame_sha256: [u8; 32],
    ) -> Self {
        Self {
            key,
            source_basis,
            prior,
            descriptor,
            reservation_record,
            reservation_frame_sha256,
        }
    }

    pub(in crate::physical_runtime) const fn key(self) -> ReleaseCustodyHeadKeyV1 {
        self.key
    }

    pub(in crate::physical_runtime) const fn source_basis(
        self,
    ) -> ReleasedGenerationReclaimBasisV1 {
        self.source_basis
    }

    pub(in crate::physical_runtime) const fn prior(self) -> Option<ReleaseCustodyHeadEntryV1> {
        self.prior
    }

    pub(in crate::physical_runtime) const fn descriptor(self) -> BlobReclaimDescriptorV3 {
        self.descriptor
    }

    pub(in crate::physical_runtime) const fn reservation_record(self) -> PersistedRecordIdentity {
        self.reservation_record
    }

    pub(in crate::physical_runtime) const fn reservation_frame_sha256(self) -> [u8; 32] {
        self.reservation_frame_sha256
    }

    /// The descriptor identity is allocated by the existing Store publication
    /// owner. The head entry can then be fixed before its WAL member is encoded.
    pub(in crate::physical_runtime) fn next_entry(
        self,
        descriptor_record: PersistedRecordIdentity,
    ) -> Result<ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadDenial> {
        let base = self.descriptor.base();
        ReleaseCustodyHeadEntryV1::new(
            self.key,
            descriptor_record,
            Sha256::digest(self.descriptor.encode()).into(),
            base.manifest_record(),
            base.manifest_frame_sha256(),
            self.reservation_record,
            self.reservation_frame_sha256,
            base.source_basis_digest(),
            base.predecessor(),
            base.source_root_generation(),
            base.cumulative_dropped(),
            base.terminal(),
        )
    }
}

impl PreparedPhysicalMutation {
    pub(in crate::physical_runtime) fn with_released_head_basis(
        mut self,
        basis: PreparedReleaseHeadBasis,
    ) -> Self {
        self.released_head_basis = Some(basis);
        self
    }

    pub(in crate::physical_runtime) const fn released_head_basis(
        &self,
    ) -> Option<PreparedReleaseHeadBasis> {
        self.released_head_basis
    }
}
