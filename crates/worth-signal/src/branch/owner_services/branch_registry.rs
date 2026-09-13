use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::state::SignalBranchId;

use super::branch_execution_cell::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};
use super::counters::SignalOwnerServiceCounters;
use super::lifecycle_state::{
    SignalOwnerAdmissionMismatch, SignalOwnerLifecycleIdentity, SignalOwnerLifecycleState,
    SignalOwnerOperationAdmission,
};
use super::operation_control::SignalOwnerOperationBoundary;

#[path = "branch_registry/retirement.rs"]
mod retirement;
pub(crate) use retirement::SignalBranchRetirement;
#[path = "branch_registry/prepared_installation.rs"]
mod prepared;
pub(crate) use prepared::{SignalPreparedBranchCell, SignalPreparedBranchInstallation};
#[path = "branch_registry/capacity.rs"]
mod capacity;
#[path = "branch_registry/close_cleanup.rs"]
mod close_cleanup;
#[path = "branch_registry/name_occupancy.rs"]
mod name_occupancy;
#[path = "branch_registry/reservation.rs"]
mod reservation;
use name_occupancy::{mark_name_installed, remove_name_for_branch};
pub(crate) use reservation::{SignalBranchOwnedReservation, SignalBranchReservation};

pub(super) struct SignalBranchCellConstruction(());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalBranchRegistryDenial {
    ForeignOwner,
    ExpiredAdmission,
    DuplicateBranch(SignalBranchId),
    UnknownBranch(SignalBranchId),
    LiveCapacityExhausted { maximum_live_branches: usize },
    ReservationCapacityExhausted { maximum_reservations: usize },
    NameAlreadyReserved,
    NameAlreadyInstalled,
    RetirementInProgress(SignalBranchId),
    ExpiredRetirement(SignalBranchId),
    TargetCellDenied(SignalBranchCellAdmissionDenial),
    OwnerMetadataOrdering,
    OwnerReentry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalBranchRegistryPoisonRecovery {
    PreservedCanonicalMembership,
}

#[derive(Debug)]
enum SignalBranchRegistryEntry<S> {
    Reserved,
    Live(Arc<SignalBranchExecutionCell<S>>),
    Retiring(Arc<SignalBranchExecutionCell<S>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignalBranchNameOccupancy {
    Reserved(SignalBranchId),
    Installed(SignalBranchId),
}

#[derive(Debug)]
struct SignalBranchRegistryState<S> {
    entries: BTreeMap<SignalBranchId, SignalBranchRegistryEntry<S>>,
    names: BTreeMap<String, SignalBranchNameOccupancy>,
    names_by_branch: BTreeMap<SignalBranchId, String>,
    live_count: usize,
    reservation_count: usize,
}

#[derive(Debug)]
pub(crate) struct SignalBranchRegistry<S> {
    owner_runtime_instance_id: u64,
    owner_lifecycle_identity: SignalOwnerLifecycleIdentity,
    maximum_live_branches: usize,
    maximum_reservations: usize,
    state: Mutex<SignalBranchRegistryState<S>>,
    counters: Arc<SignalOwnerServiceCounters>,
    recovered_poison: AtomicBool,
}

impl<S> SignalBranchRegistry<S> {
    pub(crate) fn new(
        lifecycle: &SignalOwnerLifecycleState,
        maximum_live_branches: usize,
        maximum_reservations: usize,
    ) -> Self {
        Self {
            owner_runtime_instance_id: lifecycle.owner_runtime_instance_id(),
            owner_lifecycle_identity: lifecycle.lifecycle_identity(),
            maximum_live_branches,
            maximum_reservations,
            state: Mutex::new(SignalBranchRegistryState {
                entries: BTreeMap::new(),
                names: BTreeMap::new(),
                names_by_branch: BTreeMap::new(),
                live_count: 0,
                reservation_count: 0,
            }),
            counters: lifecycle.counters(),
            recovered_poison: AtomicBool::new(false),
        }
    }

    pub(crate) fn lookup(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<Arc<SignalBranchExecutionCell<S>>, SignalBranchRegistryDenial> {
        self.validate_admission(admission)?;
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::BranchRegistryLookup);
        let _metadata_hold = admission
            .hold_owner_metadata()
            .map_err(map_metadata_hold_denial)?;
        self.counters.record_branch_registry_lookup();
        match self.lock_state().entries.get(&branch_id) {
            Some(SignalBranchRegistryEntry::Reserved) => {
                Err(SignalBranchRegistryDenial::UnknownBranch(branch_id))
            }
            Some(SignalBranchRegistryEntry::Live(cell)) => Ok(Arc::clone(cell)),
            Some(SignalBranchRegistryEntry::Retiring(_)) => {
                Err(SignalBranchRegistryDenial::RetirementInProgress(branch_id))
            }
            None => Err(SignalBranchRegistryDenial::UnknownBranch(branch_id)),
        }
    }

    #[cfg(test)]
    pub(crate) fn reserve<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<SignalBranchReservation<'a, S>, SignalBranchRegistryDenial> {
        self.reserve_entry(admission, branch_id, None)?;
        Ok(SignalBranchReservation::unnamed(self, admission, branch_id))
    }

    pub(super) fn reserve_entry(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
        name: Option<&str>,
    ) -> Result<(), SignalBranchRegistryDenial> {
        self.validate_admission(admission)?;
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::BranchRegistryReservation);
        let _metadata_hold = admission
            .hold_owner_metadata()
            .map_err(map_metadata_hold_denial)?;
        let mut state = self.lock_state();
        if let Some(name) = name {
            match state.names.get(name) {
                Some(SignalBranchNameOccupancy::Reserved(_)) => {
                    return Err(SignalBranchRegistryDenial::NameAlreadyReserved)
                }
                Some(SignalBranchNameOccupancy::Installed(_)) => {
                    return Err(SignalBranchRegistryDenial::NameAlreadyInstalled)
                }
                None => {}
            }
        }
        self.validate_available_identity(&state, branch_id)?;
        self.validate_capacity(&state)?;
        let displaced = state
            .entries
            .insert(branch_id, SignalBranchRegistryEntry::Reserved);
        debug_assert!(
            displaced.is_none(),
            "accepted reservation identity must be vacant"
        );
        if let Some(name) = name {
            let name = name.to_owned();
            let prior = state
                .names
                .insert(name.clone(), SignalBranchNameOccupancy::Reserved(branch_id));
            debug_assert!(prior.is_none());
            let prior = state.names_by_branch.insert(branch_id, name);
            debug_assert!(prior.is_none());
        }
        state.reservation_count += 1;
        self.counters.record_branch_registry_reservation();
        Ok(())
    }

    pub(crate) fn live_cells_in_identity_order(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
    ) -> Result<Vec<Arc<SignalBranchExecutionCell<S>>>, SignalBranchRegistryDenial> {
        self.validate_admission(admission)?;
        let _metadata_hold = admission
            .hold_owner_metadata()
            .map_err(map_metadata_hold_denial)?;
        let state = self.lock_state();
        let mut cells = Vec::with_capacity(state.live_count);
        for entry in state.entries.values() {
            self.counters.record_branch_registry_entry_scanned();
            if let SignalBranchRegistryEntry::Live(cell) = entry {
                cells.push(Arc::clone(cell));
            }
        }
        Ok(cells)
    }

    pub(crate) fn begin_retirement<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<SignalBranchRetirement<'a, S>, SignalBranchRegistryDenial> {
        self.validate_admission(admission)?;
        let _metadata_hold = admission
            .hold_owner_metadata()
            .map_err(map_metadata_hold_denial)?;
        let cell = {
            let mut state = self.lock_state();
            let entry = state
                .entries
                .get_mut(&branch_id)
                .ok_or(SignalBranchRegistryDenial::UnknownBranch(branch_id))?;
            let cell = match entry {
                SignalBranchRegistryEntry::Reserved => {
                    return Err(SignalBranchRegistryDenial::UnknownBranch(branch_id));
                }
                SignalBranchRegistryEntry::Live(cell) => Arc::clone(cell),
                SignalBranchRegistryEntry::Retiring(_) => {
                    return Err(SignalBranchRegistryDenial::RetirementInProgress(branch_id));
                }
            };
            *entry = SignalBranchRegistryEntry::Retiring(Arc::clone(&cell));
            cell
        };
        let retirement = SignalBranchRetirement {
            registry: self,
            admission,
            branch_id,
            cell,
            completed: false,
        };
        Ok(retirement)
    }

    pub(crate) fn poison_recovery(&self) -> Option<SignalBranchRegistryPoisonRecovery> {
        self.recovered_poison
            .load(Ordering::Acquire)
            .then_some(SignalBranchRegistryPoisonRecovery::PreservedCanonicalMembership)
    }

    fn validate_admission(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
    ) -> Result<(), SignalBranchRegistryDenial> {
        admission
            .authorize(
                self.owner_runtime_instance_id,
                self.owner_lifecycle_identity,
            )
            .map_err(SignalBranchRegistryDenial::from)
    }

    fn lock_state(&self) -> MutexGuard<'_, SignalBranchRegistryState<S>> {
        match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => {
                self.recovered_poison.store(true, Ordering::Release);
                poisoned.into_inner()
            }
        }
    }

    #[cfg(test)]
    pub(super) fn poison_state_for_test(&self) {
        let _state = self.lock_state();
        panic!("inject branch-registry poison");
    }
}

fn map_metadata_hold_denial(
    denial: super::lifecycle_state::SignalOwnerMetadataHoldDenial,
) -> SignalBranchRegistryDenial {
    match denial {
        super::lifecycle_state::SignalOwnerMetadataHoldDenial::BranchCellOrMetadataAlreadyHeld => {
            SignalBranchRegistryDenial::OwnerMetadataOrdering
        }
        super::lifecycle_state::SignalOwnerMetadataHoldDenial::ExecutingThreadReentry => {
            SignalBranchRegistryDenial::OwnerReentry
        }
    }
}

impl From<SignalOwnerAdmissionMismatch> for SignalBranchRegistryDenial {
    fn from(mismatch: SignalOwnerAdmissionMismatch) -> Self {
        match mismatch {
            SignalOwnerAdmissionMismatch::ForeignOwner => Self::ForeignOwner,
            SignalOwnerAdmissionMismatch::ExpiredLifecycle => Self::ExpiredAdmission,
        }
    }
}
