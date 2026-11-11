use std::sync::{Arc, Mutex};

use crate::branch::observation::RuntimeWorldBranchAdmissionDenial;
use crate::budget::RuntimeWorldBudgetLimit;
use crate::identity::{ProductBranchIdentity, ProductBranchIncarnation};

use super::{CustodyComponent, OwnerCreatedComponentCustodyRecord, OwnerRetirementWork};

#[derive(Debug)]
struct CustodyRegistryState {
    maximum: Option<usize>,
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
    pub(crate) fn try_has_close_custody(&self) -> Result<bool, ()> {
        let state = self.state.try_lock().map_err(|_| ())?;
        Ok(state.reserved != 0 || !state.installed.is_empty())
    }

    /// A reservation, or the product-branch incarnations its installed records
    /// belong to, for a caller that decides which incarnations are published.
    pub(crate) fn try_close_custody_incarnations(
        &self,
    ) -> Result<Option<Vec<ProductBranchIncarnation>>, ()> {
        let state = self.state.try_lock().map_err(|_| ())?;
        if state.reserved != 0 {
            return Ok(None);
        }
        let mut incarnations = Vec::new();
        incarnations
            .try_reserve_exact(state.installed.len())
            .map_err(|_| ())?;
        incarnations.extend(state.installed.iter().map(|record| record.incarnation()));
        Ok(Some(incarnations))
    }

    /// Close-time custody. A memory-only owner keeps every record. A durable
    /// Home records each published sibling in its World artifact and readmits it,
    /// so only a reservation or a record whose sibling incarnation is not an
    /// installed (published) branch remains unresolved.
    pub(crate) fn try_unresolved_close_custody(
        &self,
        durable: bool,
        branches: &crate::branch::registry::ProductBranchRegistry,
    ) -> Result<bool, ()> {
        if !durable {
            return self.try_has_close_custody();
        }
        Ok(match self.try_close_custody_incarnations()? {
            None => true,
            Some(incarnations) => incarnations
                .into_iter()
                .any(|incarnation| branches.branch_cell_by_lifecycle(incarnation).is_none()),
        })
    }

    /// Custody after a completed close. Close already refused every unpublished
    /// sibling of a durable Home, and its published siblings may since have
    /// released their live cells, so only a reservation is inconsistent there.
    pub(crate) fn try_unresolved_closed_custody(&self, durable: bool) -> Result<bool, ()> {
        if durable {
            Ok(self.try_close_custody_incarnations()?.is_none())
        } else {
            self.try_has_close_custody()
        }
    }

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
        let occupied = state
            .installed
            .len()
            .checked_add(state.reserved)
            .ok_or(RuntimeWorldBranchAdmissionDenial::CustodyCapacityExhausted)?;
        if state.maximum.is_some_and(|maximum| occupied >= maximum) {
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
