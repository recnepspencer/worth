use worth_store_physical_format::store_namespace::StableStoreIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecoveryAllocationAdmission {
    store: StableStoreIdentity,
    byte_limit: u64,
}

impl PhysicalRecoveryAllocationAdmission {
    pub(in crate::physical_runtime) const fn new(
        store: StableStoreIdentity,
        byte_limit: u64,
    ) -> Self {
        Self { store, byte_limit }
    }

    pub const fn store_identity(self) -> StableStoreIdentity {
        self.store
    }

    pub const fn byte_limit(self) -> u64 {
        self.byte_limit
    }
}
#[cfg(test)]
#[path = "recovery_allocation_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
