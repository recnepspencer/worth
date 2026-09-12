use worth_store_security::{StoreSecurityMetadata, StoreSecurityScopePropagationWitness};

use super::{
    LogicalDecodeSecurityScopeEntry, StableReadSecurityScopeCarrierBasis,
    StableReadSecurityScopePropagationCounters,
};
use crate::physical_runtime::stability::PhysicalByteGuardScope;
use worth_store_physical_isolation::{CurrentPhysicalRoot, PhysicalReadProtectedFootprintBasis};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StableReadObservedSecurityScope {
    observed_root: CurrentPhysicalRoot,
    footprint_basis: PhysicalReadProtectedFootprintBasis,
    carrier_basis: StableReadSecurityScopeCarrierBasis,
    witness: StoreSecurityScopePropagationWitness,
    counters: StableReadSecurityScopePropagationCounters,
}

impl StableReadObservedSecurityScope {
    pub(crate) const fn new(
        observed_root: CurrentPhysicalRoot,
        footprint_basis: PhysicalReadProtectedFootprintBasis,
        carrier_basis: StableReadSecurityScopeCarrierBasis,
        witness: StoreSecurityScopePropagationWitness,
        counters: StableReadSecurityScopePropagationCounters,
    ) -> Self {
        Self {
            observed_root,
            footprint_basis,
            carrier_basis,
            witness,
            counters,
        }
    }

    pub const fn logical_decode_entry_scope(self) -> LogicalDecodeSecurityScopeEntry {
        LogicalDecodeSecurityScopeEntry::from_observed_scope(
            self.observed_root,
            self.footprint_basis,
            self.carrier_basis,
            self.witness,
            self.counters.with_logical_decode_entry(),
        )
    }

    pub const fn observed_root(self) -> CurrentPhysicalRoot {
        self.observed_root
    }

    pub const fn footprint_basis(self) -> PhysicalReadProtectedFootprintBasis {
        self.footprint_basis
    }

    pub const fn guard_scope(self) -> PhysicalByteGuardScope {
        self.carrier_basis.guard_scope()
    }

    pub const fn carrier_basis(self) -> StableReadSecurityScopeCarrierBasis {
        self.carrier_basis
    }

    pub const fn metadata(self) -> StoreSecurityMetadata {
        self.witness.metadata()
    }

    pub const fn counters(self) -> StableReadSecurityScopePropagationCounters {
        self.counters
    }
}
