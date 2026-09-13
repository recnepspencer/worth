use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use worth_runtime_world::facade::{
    ProductBranchIdentity, ProductBranchIncarnation, ProductBranchObservation,
    RuntimeWorldBudgetLimit,
};

use super::{
    capacity::ActivationCapacity, gate::WorthQueryProductActivationGate,
    WorthQueryProductActivationDenial,
};

struct ActivationEntry {
    incarnation: ProductBranchIncarnation,
    gate: Arc<WorthQueryProductActivationGate>,
}

/// Bounded coordination only. Entries contain no head or component selection.
pub(crate) struct WorthQueryProductActivationRegistry {
    gates: Mutex<HashMap<ProductBranchIdentity, ActivationEntry>>,
    capacity: Arc<ActivationCapacity>,
}

impl WorthQueryProductActivationRegistry {
    pub(crate) fn new(
        limit: RuntimeWorldBudgetLimit,
    ) -> Result<Self, WorthQueryProductActivationDenial> {
        let mut gates = HashMap::new();
        gates
            .try_reserve(limit.get())
            .map_err(|_| WorthQueryProductActivationDenial::AllocationRejected)?;
        Ok(Self {
            gates: Mutex::new(gates),
            capacity: ActivationCapacity::new(limit.get()),
        })
    }

    pub(crate) fn reserve(
        &self,
    ) -> Result<WorthQueryProductActivationReservation<'_>, WorthQueryProductActivationDenial> {
        let capacity = self.capacity.reserve()?;
        Ok(WorthQueryProductActivationReservation {
            registry: self,
            gate: Arc::new(WorthQueryProductActivationGate::new(capacity)),
        })
    }

    pub(crate) fn gate(
        &self,
        identity: &ProductBranchIdentity,
    ) -> Result<Arc<WorthQueryProductActivationGate>, WorthQueryProductActivationDenial> {
        self.gates
            .lock()
            .map_err(|_| WorthQueryProductActivationDenial::RegistryUnavailable)?
            .get(identity)
            .map(|entry| Arc::clone(&entry.gate))
            .ok_or(WorthQueryProductActivationDenial::UnknownProductBranch)
    }

    #[cfg(feature = "test-world-operation-control")]
    pub(crate) fn installed_branch_count(&self) -> usize {
        self.gates
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// Release only the retired occurrence; delayed cleanup cannot erase a replacement.
    pub(crate) fn release(
        &self,
        identity: &ProductBranchIdentity,
        incarnation: ProductBranchIncarnation,
    ) -> bool {
        let mut gates = self
            .gates
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if gates
            .get(identity)
            .is_some_and(|entry| entry.incarnation == incarnation)
        {
            gates.remove(identity);
            true
        } else {
            false
        }
    }
}

/// All fallible storage admission precedes World branch creation.
pub(crate) struct WorthQueryProductActivationReservation<'a> {
    registry: &'a WorthQueryProductActivationRegistry,
    gate: Arc<WorthQueryProductActivationGate>,
}

impl WorthQueryProductActivationReservation<'_> {
    pub(crate) fn commit(self, branch: &ProductBranchObservation) {
        self.registry
            .gates
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                branch.branch_identity().clone(),
                ActivationEntry {
                    incarnation: branch.lifecycle_incarnation(),
                    gate: self.gate,
                },
            );
    }
}
