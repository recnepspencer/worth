use std::collections::{HashMap, HashSet};
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
    state: Mutex<ActivationRegistryState>,
    capacity: Arc<ActivationCapacity>,
}

struct ActivationRegistryState {
    gates: HashMap<ProductBranchIdentity, ActivationEntry>,
    live_occurrences: HashSet<ProductBranchIncarnation>,
}

impl WorthQueryProductActivationRegistry {
    pub(crate) fn new(
        limit: RuntimeWorldBudgetLimit,
    ) -> Result<Self, WorthQueryProductActivationDenial> {
        let mut gates = HashMap::new();
        gates
            .try_reserve(limit.get())
            .map_err(|_| WorthQueryProductActivationDenial::AllocationRejected)?;
        let mut live_occurrences = HashSet::new();
        live_occurrences
            .try_reserve(limit.get())
            .map_err(|_| WorthQueryProductActivationDenial::AllocationRejected)?;
        Ok(Self {
            state: Mutex::new(ActivationRegistryState {
                gates,
                live_occurrences,
            }),
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
        self.state
            .lock()
            .map_err(|_| WorthQueryProductActivationDenial::RegistryUnavailable)?
            .gates
            .get(identity)
            .map(|entry| Arc::clone(&entry.gate))
            .ok_or(WorthQueryProductActivationDenial::UnknownProductBranch)
    }

    pub(crate) fn first_unadmitted_live_occurrence(
        &self,
        occurrences: impl IntoIterator<Item = ProductBranchIncarnation>,
    ) -> Result<Option<ProductBranchIncarnation>, WorthQueryProductActivationDenial> {
        let state = self
            .state
            .lock()
            .map_err(|_| WorthQueryProductActivationDenial::RegistryUnavailable)?;
        Ok(occurrences
            .into_iter()
            .find(|occurrence| !state.live_occurrences.contains(occurrence)))
    }

    #[cfg(feature = "test-world-operation-control")]
    pub(crate) fn installed_branch_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .gates
            .len()
    }

    /// Release only the retired occurrence; delayed cleanup cannot erase a replacement.
    pub(crate) fn release(
        &self,
        identity: &ProductBranchIdentity,
        incarnation: ProductBranchIncarnation,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state
            .gates
            .get(identity)
            .is_some_and(|entry| entry.incarnation == incarnation)
        {
            state.gates.remove(identity);
            state.live_occurrences.remove(&incarnation);
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
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let replaced = state.gates.insert(
            branch.branch_identity().clone(),
            ActivationEntry {
                incarnation: branch.lifecycle_incarnation(),
                gate: self.gate,
            },
        );
        if let Some(replaced) = replaced {
            state.live_occurrences.remove(&replaced.incarnation);
        }
        state
            .live_occurrences
            .insert(branch.lifecycle_incarnation());
    }
}
