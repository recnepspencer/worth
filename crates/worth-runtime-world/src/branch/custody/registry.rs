use std::sync::{Arc, Mutex};

use crate::branch::observation::RuntimeWorldBranchAdmissionDenial;
use crate::budget::RuntimeWorldBudgetLimit;
use crate::identity::{ProductBranchIdentity, ProductBranchIncarnation};

use super::{CustodyComponent, OwnerCreatedComponentCustodyRecord};

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

    /// Drain the records one exact product-branch occurrence created. The key
    /// is the identity **and** its incarnation: a name-keyed identity outlives
    /// retirement, so filtering on it alone would hand a recreated branch the
    /// component branches an earlier occurrence created.
    pub(crate) fn take_for_incarnation(
        &self,
        branch: &ProductBranchIdentity,
        incarnation: ProductBranchIncarnation,
    ) -> Vec<OwnerCreatedComponentCustodyRecord> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let mut taken = Vec::new();
        let mut retained = Vec::with_capacity(state.installed.len() + state.reserved);
        for record in std::mem::take(&mut state.installed) {
            if record.product_branch() == branch && record.incarnation() == incarnation {
                taken.push(record);
            } else {
                retained.push(record);
            }
        }
        state.installed = retained;
        taken
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
