use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::{
    WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    WorthQueryProductBranchOwnerCleanupDenial, WorthQueryProductBranchOwnerCleanupReceipt,
    WorthQueryProductBranchOwnerCleanupRecord, WorthQueryProductBranchOwnerCleanupWork,
    WorthQueryRetiredProductOccurrence,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;

struct CleanupEntry {
    record: Mutex<Option<WorthQueryProductBranchOwnerCleanupRecord>>,
    scope: CleanupScope,
}

#[derive(Clone)]
pub(crate) enum CleanupScope {
    Creation,
    Application,
    WorkspaceRetirement,
    ApplicationRetirement(WorthQueryRetiredProductOccurrence),
}

struct CleanupRegistryState {
    maximum: usize,
    next_identity: u64,
    reserved: usize,
    entries: HashMap<u64, Arc<CleanupEntry>>,
}

#[derive(Clone)]
pub(crate) struct WorthQueryProductBranchOwnerCleanupRegistry {
    state: Arc<Mutex<CleanupRegistryState>>,
}

impl WorthQueryProductBranchOwnerCleanupRegistry {
    pub(crate) fn new(maximum: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(CleanupRegistryState {
                maximum,
                next_identity: 1,
                reserved: 0,
                entries: HashMap::new(),
            })),
        }
    }

    pub(super) fn reserve(
        &self,
        scope: CleanupScope,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    > {
        let entry = Arc::new(CleanupEntry {
            record: Mutex::new(None),
            scope,
        });
        let mut state = lock(&self.state);
        if state.entries.len().saturating_add(state.reserved) >= state.maximum {
            return Err(WorthQueryProductBranchOwnerCleanupCapacityExhausted);
        }
        let identity = state.next_identity;
        state.next_identity = state
            .next_identity
            .checked_add(1)
            .ok_or(WorthQueryProductBranchOwnerCleanupCapacityExhausted)?;
        let additional_capacity = state
            .reserved
            .checked_add(1)
            .ok_or(WorthQueryProductBranchOwnerCleanupCapacityExhausted)?;
        state
            .entries
            .try_reserve(additional_capacity)
            .map_err(|_| WorthQueryProductBranchOwnerCleanupCapacityExhausted)?;
        state.reserved += 1;
        Ok(WorthQueryProductBranchOwnerCleanupReservation {
            registry: self.clone(),
            identity,
            entry: Some(entry),
            armed: true,
        })
    }

    pub(super) fn pending_identities(&self) -> Vec<u64> {
        let mut identities: Vec<_> = lock(&self.state).entries.keys().copied().collect();
        identities.sort_unstable();
        identities
    }

    pub(super) fn pending_workspace_identities(&self) -> Vec<u64> {
        let mut identities: Vec<_> = lock(&self.state)
            .entries
            .iter()
            .filter_map(|(identity, entry)| {
                matches!(
                    entry.scope,
                    CleanupScope::Creation | CleanupScope::WorkspaceRetirement
                )
                .then_some(*identity)
            })
            .collect();
        identities.sort_unstable();
        identities
    }

    pub(super) fn pending_work(
        &self,
        identity: u64,
    ) -> Vec<WorthQueryProductBranchOwnerCleanupWork> {
        let entry = lock(&self.state).entries.get(&identity).cloned();
        let Some(entry) = entry else {
            return Vec::new();
        };
        let pending = lock(&entry.record)
            .as_ref()
            .map(WorthQueryProductBranchOwnerCleanupRecord::described_work)
            .unwrap_or_default();
        pending
    }

    pub(super) fn pending_signal_references(
        &self,
        identity: u64,
    ) -> Vec<worth_signal::facade::branch::ManagedSignalBranchReference> {
        let entry = lock(&self.state).entries.get(&identity).cloned();
        let Some(entry) = entry else {
            return Vec::new();
        };
        let references = lock(&entry.record)
            .as_ref()
            .map(|record| {
                record
                    .pending
                    .iter()
                    .filter_map(|work| match work {
                        worth_runtime_world::facade::OwnerRetirementWork::SignalBranchRetirement {
                            reference,
                            ..
                        } => Some(reference.clone()),
                        worth_runtime_world::facade::OwnerRetirementWork::RelationalBranchRetirement {
                            ..
                        } => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        references
    }

    pub(super) fn release_history(
        &self,
        identity: u64,
        runtime: &WorthQueryProductRuntime,
    ) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial> {
        let entry = lock(&self.state)
            .entries
            .get(&identity)
            .cloned()
            .ok_or(WorthQueryProductBranchOwnerCleanupDenial::CleanupUnavailable)?;
        let mut record = RetryRecordGuard::take(&entry.record)?;
        record.record_mut().release_retired_history(runtime)
    }

    pub(super) fn pending_application_retired_product_occurrences(
        &self,
    ) -> Vec<WorthQueryRetiredProductOccurrence> {
        lock(&self.state)
            .entries
            .values()
            .filter_map(|entry| match &entry.scope {
                CleanupScope::ApplicationRetirement(occurrence) => Some(occurrence.clone()),
                CleanupScope::Creation
                | CleanupScope::Application
                | CleanupScope::WorkspaceRetirement => None,
            })
            .collect()
    }

    pub(super) fn retry(
        &self,
        identity: u64,
        runtime: &WorthQueryProductRuntime,
    ) -> Result<WorthQueryProductBranchOwnerCleanupReceipt, WorthQueryProductBranchOwnerCleanupDenial>
    {
        let entry = lock(&self.state)
            .entries
            .get(&identity)
            .cloned()
            .ok_or(WorthQueryProductBranchOwnerCleanupDenial::CleanupUnavailable)?;
        let mut record = RetryRecordGuard::take(&entry.record)?;
        match record
            .record_mut()
            .retry(runtime, || self.after_component_progress())
        {
            Ok(receipt) => {
                let mut state = lock(&self.state);
                if state
                    .entries
                    .get(&identity)
                    .is_some_and(|installed| Arc::ptr_eq(installed, &entry))
                {
                    state.entries.remove(&identity);
                }
                drop(state);
                record.complete();
                Ok(receipt)
            }
            Err(denial) => Err(denial),
        }
    }

    #[cfg(test)]
    fn after_component_progress(&self) {
        testing::run_after_component_progress();
    }

    #[cfg(not(test))]
    fn after_component_progress(&self) {}

    #[cfg(test)]
    fn after_install(&self) {
        testing::run_after_install();
    }

    #[cfg(not(test))]
    fn after_install(&self) {}

    #[cfg(test)]
    pub(crate) fn replace_after_install_hook(&self, hook: Option<Box<dyn Fn()>>) {
        testing::replace_after_install_hook(hook);
    }

    #[cfg(test)]
    pub(crate) fn replace_after_component_progress_hook(&self, hook: Option<Box<dyn Fn()>>) {
        testing::replace_after_component_progress_hook(hook);
    }
}

#[must_use = "cleanup capacity must be installed or released"]
pub(crate) struct WorthQueryProductBranchOwnerCleanupReservation {
    registry: WorthQueryProductBranchOwnerCleanupRegistry,
    identity: u64,
    entry: Option<Arc<CleanupEntry>>,
    armed: bool,
}

impl WorthQueryProductBranchOwnerCleanupReservation {
    pub(crate) fn install_retirement(
        self,
        report: worth_runtime_world::facade::ProductBranchRetirementReport,
    ) -> u64 {
        self.install(WorthQueryProductBranchOwnerCleanupRecord::from_retirement(
            report,
        ))
    }

    pub(crate) fn install_unpublished(
        self,
        cleanup: worth_runtime_world::facade::ProductUnpublishedCleanup,
    ) -> u64 {
        self.install(WorthQueryProductBranchOwnerCleanupRecord::from_unpublished(
            cleanup,
        ))
    }

    fn install(mut self, record: WorthQueryProductBranchOwnerCleanupRecord) -> u64 {
        let entry = self
            .entry
            .take()
            .expect("a cleanup reservation owns its preallocated entry");
        *lock(&entry.record) = Some(record);
        let mut state = lock(&self.registry.state);
        state.reserved = state
            .reserved
            .checked_sub(1)
            .expect("a cleanup reservation owns one slot");
        let replaced = state.entries.insert(self.identity, entry);
        assert!(replaced.is_none(), "cleanup identities install once");
        self.armed = false;
        self.registry.after_install();
        self.identity
    }
}

#[path = "registry/retry.rs"]
mod retry;
use retry::RetryRecordGuard;
#[cfg(test)]
#[path = "registry/testing.rs"]
mod testing;

impl Drop for WorthQueryProductBranchOwnerCleanupReservation {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut state = lock(&self.registry.state);
        state.reserved = state
            .reserved
            .checked_sub(1)
            .expect("a cleanup reservation owns one slot");
        self.armed = false;
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}
