use worth_store_security::{StoreSecurityMetadata, StoreSecurityScopePropagationWitness};

use super::{StableReadSecurityScopeCarrierBasis, StableReadSecurityScopePropagationCounters};
use crate::physical_runtime::stability::PhysicalByteGuardScope;
use worth_store_physical_isolation::{CurrentPhysicalRoot, PhysicalReadProtectedFootprintBasis};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalDecodeSecurityScopeEntry {
    observed_root: CurrentPhysicalRoot,
    footprint_basis: PhysicalReadProtectedFootprintBasis,
    carrier_basis: StableReadSecurityScopeCarrierBasis,
    witness: StoreSecurityScopePropagationWitness,
    counters: StableReadSecurityScopePropagationCounters,
}

impl LogicalDecodeSecurityScopeEntry {
    pub(crate) const fn from_observed_scope(
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
