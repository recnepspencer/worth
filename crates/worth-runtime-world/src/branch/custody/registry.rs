use std::sync::{Arc, Mutex};

use crate::branch::observation::RuntimeWorldBranchAdmissionDenial;
use crate::budget::RuntimeWorldBudgetLimit;
use crate::identity::{ProductBranchIdentity, ProductBranchIncarnation};

use super::{CustodyComponent, OwnerCreatedComponentCustodyRecord, OwnerRetirementWork};

#[derive(Debug)]
struct CustodyRegistryState {
    maximum: usize,
    reserved: usize,
    installed: Vec<OwnerCreatedComponentCustodyRecord>,
}

/// The only managed registry of owner-created component branches. It is
/// bounded by the installed custody budget and charged before the owner fork
/// that would create the branch it records.
#[derive(Debug, Clone)]
pub(crate) struct OwnerCreatedComponentCustodyRegistry {
    state: Arc<Mutex<CustodyRegistryState>>,
}

impl OwnerCreatedComponentCustodyRegistry {
    pub(crate) fn new(maximum: RuntimeWorldBudgetLimit) -> Self {
        Self {
            state: Arc::new(Mutex::new(CustodyRegistryState {
                maximum: maximum.get(),
                reserved: 0,
                installed: Vec::new(),
            })),
        }
    }

    /// Charged before the owner fork that would create the recorded branch;
    /// exhaustion denies pre-effect.
    pub(crate) fn reserve(
        &self,
        component: CustodyComponent,
    ) -> Result<ReservedCustodySlot, RuntimeWorldBranchAdmissionDenial> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.installed.len().saturating_add(state.reserved) >= state.maximum {
            return Err(RuntimeWorldBranchAdmissionDenial::CustodyCapacityExhausted);
        }
        let reserved = state
            .reserved
            .checked_add(1)
            .ok_or(RuntimeWorldBranchAdmissionDenial::CustodyCapacityExhausted)?;
        state
            .installed
            .try_reserve(reserved)
            .map_err(|_| RuntimeWorldBranchAdmissionDenial::CustodyCapacityExhausted)?;
        state.reserved = reserved;
        drop(state);
        Ok(ReservedCustodySlot {
            registry: self.clone(),
            component,
            armed: true,
        })
    }

    /// Reserve the exact output needed to drain one occurrence before the
    /// occurrence or a recovery record naming it is removed.
    pub(crate) fn reserve_retirement_work(
        &self,
        branch: &ProductBranchIdentity,
        incarnation: ProductBranchIncarnation,
    ) -> Result<Vec<OwnerRetirementWork>, ()> {
        let count = self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .installed
            .iter()
            .filter(|record| {
                record.product_branch() == branch && record.incarnation() == incarnation
            })
            .count();
        let mut work = Vec::new();
        work.try_reserve_exact(count).map_err(|_| ())?;
        Ok(work)
    }

    /// Drain the records one exact product-branch occurrence created into
    /// storage reserved before the authority naming that occurrence moved.
    pub(crate) fn drain_retirement_work_into(
        &self,
        branch: &ProductBranchIdentity,
        incarnation: ProductBranchIncarnation,
        work: &mut Vec<OwnerRetirementWork>,
    ) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let mut index = 0;
        while index < state.installed.len() {
            if state.installed[index].product_branch() == branch
                && state.installed[index].incarnation() == incarnation
            {
                let record = state.installed.swap_remove(index);
                work.push(record.into_retirement_work());
            } else {
                index += 1;
            }
        }
    }

    /// Take every record still charged against this registry. Close drains
    /// custody exactly once and reports the typed work it drained; a record
    /// left installed behind a closed owner would be a component branch nobody
    /// is named as owing, so the drain empties the registry rather than
    /// reading it.
    pub(crate) fn take_all_installed(&self) -> Vec<OwnerCreatedComponentCustodyRecord> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        std::mem::take(&mut state.installed)
    }

    /// Every record still charged against this registry, in installation order.
    #[cfg(test)]
    pub(crate) fn installed_records(&self) -> Vec<OwnerCreatedComponentCustodyRecord> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .installed
            .clone()
    }

    #[cfg(test)]
    pub(crate) fn installed(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .installed
            .len()
    }
}

/// One charged custody slot. Dropping it uninstalled releases its charge.
#[must_use = "a reserved custody slot is installed or dropped"]
pub(crate) struct ReservedCustodySlot {
    registry: OwnerCreatedComponentCustodyRegistry,
    component: CustodyComponent,
    armed: bool,
}

impl std::fmt::Debug for ReservedCustodySlot {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReservedCustodySlot")
            .field("component", &self.component)
            .field("armed", &self.armed)
            .finish_non_exhaustive()
    }
}

impl ReservedCustodySlot {
    pub(crate) fn install(mut self, record: OwnerCreatedComponentCustodyRecord) {
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.reserved = state
            .reserved
            .checked_sub(1)
            .expect("a live custody reservation owns one slot");
        state.installed.push(record);
        drop(state);
        self.armed = false;
    }
}

impl Drop for ReservedCustodySlot {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.reserved = state
            .reserved
            .checked_sub(1)
            .expect("a live custody reservation owns one slot");
        self.armed = false;
    }
}
