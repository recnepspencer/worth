use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
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
    unavailable_programs: HashSet<ApplicationProgramRevision>,
    in_flight_programs: HashMap<ApplicationProgramRevision, usize>,
    program_coordination_required: bool,
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
                unavailable_programs: HashSet::new(),
                in_flight_programs: HashMap::new(),
                program_coordination_required: false,
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
            program: None,
            program_reservation_active: false,
        })
    }

    pub(crate) fn reserve_for_source_program(
        &self,
        program: Option<&ApplicationProgramRevision>,
    ) -> Result<WorthQueryProductActivationReservation<'_>, WorthQueryProductActivationDenial> {
        let capacity = self.capacity.reserve()?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| WorthQueryProductActivationDenial::RegistryUnavailable)?;
        if state.program_coordination_required && program.is_none() {
            return Err(WorthQueryProductActivationDenial::ProgramSupportUnavailable);
        }
        if program.is_some_and(|revision| state.unavailable_programs.contains(revision)) {
            return Err(WorthQueryProductActivationDenial::ProgramSupportUnavailable);
        }
        if let Some(revision) = program {
            let active = state
                .in_flight_programs
                .entry(revision.clone())
                .or_default();
            *active = active
                .checked_add(1)
                .ok_or(WorthQueryProductActivationDenial::CapacityExhausted)?;
        }
        drop(state);
        Ok(WorthQueryProductActivationReservation {
            registry: self,
            gate: Arc::new(WorthQueryProductActivationGate::new(capacity)),
            program: program.cloned(),
            program_reservation_active: true,
        })
    }

    pub(crate) fn require_program_coordination(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.program_coordination_required = true;
    }

    pub(crate) fn begin_program_retirement(
        self: &Arc<Self>,
        revision: &ApplicationProgramRevision,
    ) -> Result<WorthQueryProgramRetirementBarrier, WorthQueryProductActivationDenial> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| WorthQueryProductActivationDenial::RegistryUnavailable)?;
        if state
            .in_flight_programs
            .get(revision)
            .copied()
            .unwrap_or_default()
            != 0
        {
            return Err(WorthQueryProductActivationDenial::PublicationInProgress);
        }
        if !state.unavailable_programs.insert(revision.clone()) {
            return Err(WorthQueryProductActivationDenial::ProgramSupportUnavailable);
        }
        Ok(WorthQueryProgramRetirementBarrier {
            registry: Arc::clone(self),
            revision: revision.clone(),
            committed: false,
        })
    }

    pub(crate) fn live_occurrences(
        &self,
    ) -> Result<Box<[ProductBranchIncarnation]>, WorthQueryProductActivationDenial> {
        let state = self
            .state
            .lock()
            .map_err(|_| WorthQueryProductActivationDenial::RegistryUnavailable)?;
        let mut occurrences = Vec::new();
        occurrences
            .try_reserve_exact(state.live_occurrences.len())
            .map_err(|_| WorthQueryProductActivationDenial::AllocationRejected)?;
        occurrences.extend(state.live_occurrences.iter().copied());
        occurrences.sort_unstable();
        Ok(occurrences.into_boxed_slice())
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
    program: Option<ApplicationProgramRevision>,
    program_reservation_active: bool,
}

impl WorthQueryProductActivationReservation<'_> {
    pub(crate) fn commit(mut self, branch: &ProductBranchObservation) {
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let replaced = state.gates.insert(
            branch.branch_identity().clone(),
            ActivationEntry {
                incarnation: branch.lifecycle_incarnation(),
                gate: Arc::clone(&self.gate),
            },
        );
        if let Some(replaced) = replaced {
            state.live_occurrences.remove(&replaced.incarnation);
        }
        state
            .live_occurrences
            .insert(branch.lifecycle_incarnation());
        self.release_program_reservation(&mut state);
    }

    fn release_program_reservation(&mut self, state: &mut ActivationRegistryState) {
        if !self.program_reservation_active {
            return;
        }
        if let Some(revision) = &self.program {
            let remove = {
                let active = state
                    .in_flight_programs
                    .get_mut(revision)
                    .expect("a source reservation owns one in-flight program use");
                *active = active.checked_sub(1).expect("program use cannot underflow");
                *active == 0
            };
            if remove {
                state.in_flight_programs.remove(revision);
            }
        }
        self.program_reservation_active = false;
    }
}

impl Drop for WorthQueryProductActivationReservation<'_> {
    fn drop(&mut self) {
        if !self.program_reservation_active {
            return;
        }
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.release_program_reservation(&mut state);
    }
}

pub(crate) struct WorthQueryProgramRetirementBarrier {
    registry: Arc<WorthQueryProductActivationRegistry>,
    revision: ApplicationProgramRevision,
    committed: bool,
}

impl WorthQueryProgramRetirementBarrier {
    pub(crate) fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for WorthQueryProgramRetirementBarrier {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        self.registry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .unavailable_programs
            .remove(&self.revision);
    }
}
