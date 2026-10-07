/// A complete residency policy admitted against one physical record format.
///
/// Store retains this sealed value when it constructs the instance's single
/// buffer pool. Its getters are configuration evidence, not allocation,
/// eviction, or retry authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmittedPhysicalRecordResidencyPolicy {
    pub(super) limits: worth_store_buffer_pool::PhysicalResidencyLimits,
    pub(super) record_format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
}

impl AdmittedPhysicalRecordResidencyPolicy {
    /// The same complete default used by ordinary Record initialization and open.
    pub fn canonical(
        format: crate::physical_runtime::record_serving::AdmittedPhysicalRecordFormat,
    ) -> Self {
        super::defaults::canonical_residency_policy(format)
    }

    /// Entry-binding identity of the complete admitted policy, not allocation authority.
    pub fn canonical_identity_bytes(self) -> [u8; 32] {
        use super::{
            PhysicalOperationAllocationScope as Scope, PhysicalSpeculativeWorkKind as Kind,
        };
        use sha2::{Digest, Sha256};

        let mut digest = Sha256::new();
        digest.update(b"worth.store.physical.residency.policy@1");
        digest.update(self.record_format.canonical_identity_bytes());
        for bytes in [
            self.total_bytes(),
            self.resident_bytes(),
            self.metadata_bytes(),
            u64::from(self.frame_entries()),
            u64::from(self.pinned_frames()),
            u64::from(self.pin_leases()),
            u64::from(self.dirty_frames()),
            self.dirty_replacement_bytes(),
            self.operation_bytes(),
        ] {
            digest.update(bytes.to_le_bytes());
        }
        for scope in [
            Scope::ForegroundRead,
            Scope::ForegroundWrite,
            Scope::Recovery,
            Scope::Scrub,
            Scope::Maintenance,
            Scope::Verification,
            Scope::Blob,
        ] {
            digest.update(self.scope_bytes(scope).to_le_bytes());
        }
        for kind in [Kind::ReadAhead, Kind::Prefetch, Kind::WriteBehind] {
            digest.update(self.speculative_frames(kind).to_le_bytes());
        }
        digest.update(self.limits.progress_headroom_bytes().to_le_bytes());
        digest.finalize().into()
    }

    /// The physical geometry against which every dimension was admitted.
    pub const fn record_format(
        self,
    ) -> worth_store_physical_format::PhysicalRecordFormatDeclaration {
        self.record_format
    }

    pub fn matches_format(
        self,
        format: crate::physical_runtime::record_serving::AdmittedPhysicalRecordFormat,
    ) -> bool {
        self.record_format == format.declaration()
    }

    pub(in crate::physical_runtime) const fn limits(
        self,
    ) -> worth_store_buffer_pool::PhysicalResidencyLimits {
        self.limits
    }

    /// Returns the hard envelope for all live pool-owned bytes.
    pub const fn total_bytes(self) -> u64 {
        self.limits.total_bytes()
    }

    /// Returns the resident frame-payload byte ceiling.
    pub const fn resident_bytes(self) -> u64 {
        self.limits.resident_bytes()
    }

    /// Returns the frame-table metadata byte ceiling.
    pub const fn metadata_bytes(self) -> u64 {
        self.limits.metadata_bytes()
    }

    /// Returns the frame identity ceiling.
    pub const fn frame_entries(self) -> u32 {
        self.limits.frame_entries()
    }

    /// Returns the simultaneously pinned frame ceiling.
    pub const fn pinned_frames(self) -> u32 {
        self.limits.pinned_frames()
    }

    /// Returns the live pin-lease ceiling.
    pub const fn pin_leases(self) -> u32 {
        self.limits.pin_leases()
    }

    /// Returns the dirty frame ceiling.
    pub const fn dirty_frames(self) -> u32 {
        self.limits.dirty_frames()
    }

    /// Returns the dirty replacement byte ceiling.
    pub const fn dirty_replacement_bytes(self) -> u64 {
        self.limits.dirty_replacement_bytes()
    }

    /// Returns the aggregate operation-owned byte ceiling.
    pub const fn operation_bytes(self) -> u64 {
        self.limits.operation_bytes()
    }

    /// Returns one operation scope's byte ceiling.
    pub const fn scope_bytes(self, scope: super::PhysicalOperationAllocationScope) -> u64 {
        self.limits.scope_bytes(scope)
    }

    /// Returns one speculative work kind's frame ceiling.
    pub const fn speculative_frames(self, kind: super::PhysicalSpeculativeWorkKind) -> u32 {
        self.limits.speculative_frames(kind)
    }
}
