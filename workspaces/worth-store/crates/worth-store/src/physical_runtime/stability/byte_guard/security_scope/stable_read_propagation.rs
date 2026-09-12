use worth_proof::TransitionOutcome;
use worth_store_security::{
    deny_stale_store_security_scope, propagate_store_security_scope, StoreSecurityMetadata,
    StoreSecurityScopePropagationSite, StoreSecurityScopePropagationWitness,
};

use super::{
    StableReadObservedSecurityScope, StableReadSecurityScopeCarrierBasis,
    StableReadSecurityScopePropagationCounters, StableReadSecurityScopePropagationDenial,
    StableReadSecurityScopePropagationInput,
};
use worth_store_physical_isolation::{CurrentPhysicalRoot, PhysicalReadProtectedFootprintBasis};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StableReadSecurityScopePropagation {
    protected_root: CurrentPhysicalRoot,
    footprint_basis: PhysicalReadProtectedFootprintBasis,
    carrier_basis: StableReadSecurityScopeCarrierBasis,
    witness: StoreSecurityScopePropagationWitness,
    counters: StableReadSecurityScopePropagationCounters,
}

impl StableReadSecurityScopePropagation {
    pub fn protect(
        input: StableReadSecurityScopePropagationInput,
    ) -> TransitionOutcome<Self, StableReadSecurityScopePropagationDenial> {
        match propagate_store_security_scope(
            input.manifest_metadata(),
            input.page_metadata(),
            StoreSecurityScopePropagationSite::StableReadProtection,
        ) {
            TransitionOutcome::Success(witness) => TransitionOutcome::success(Self::from_witness(
                input.protected_root(),
                input.footprint_basis(),
                input.carrier_basis(),
                witness,
            )),
            TransitionOutcome::Denied(denial) => TransitionOutcome::denied(
                StableReadSecurityScopePropagationDenial::from_store_denial(denial),
            ),
            TransitionOutcome::Deferred(deferred) => match deferred {},
            TransitionOutcome::Stale(stale) => match stale {},
            TransitionOutcome::RebindRequired(rebind) => match rebind {},
            TransitionOutcome::Failed(failed) => match failed {},
        }
    }

    pub fn observe_after_root_check(
        self,
        observed_root: CurrentPhysicalRoot,
    ) -> TransitionOutcome<StableReadObservedSecurityScope, StableReadSecurityScopePropagationDenial>
    {
        if self.protected_root.epoch().get() != observed_root.epoch().get()
            || self.protected_root.manifest_epoch().get() != observed_root.manifest_epoch().get()
        {
            return TransitionOutcome::denied(
                StableReadSecurityScopePropagationDenial::from_store_denial(
                    deny_stale_store_security_scope(
                        StoreSecurityScopePropagationSite::StableReadRootObservation,
                    ),
                ),
            );
        }

        TransitionOutcome::success(StableReadObservedSecurityScope::new(
            observed_root,
            self.footprint_basis,
            self.carrier_basis,
            self.witness,
            self.counters.with_root_observation(),
        ))
    }

    pub const fn metadata(self) -> StoreSecurityMetadata {
        self.witness.metadata()
    }

    pub const fn counters(self) -> StableReadSecurityScopePropagationCounters {
        self.counters
    }

    const fn from_witness(
        protected_root: CurrentPhysicalRoot,
        footprint_basis: PhysicalReadProtectedFootprintBasis,
        carrier_basis: StableReadSecurityScopeCarrierBasis,
        witness: StoreSecurityScopePropagationWitness,
    ) -> Self {
        Self {
            protected_root,
            footprint_basis,
            carrier_basis,
            witness,
            counters: StableReadSecurityScopePropagationCounters::from_store_counters(
                witness.counters(),
            ),
        }
    }
}
