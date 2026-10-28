use std::sync::atomic::{AtomicBool, Ordering};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, ExecutionUniqueOrderedMap};
mod reservation;
pub(crate) use reservation::{
    WorthQueryCleanupReservationDenial, WorthQueryProductBranchOwnerCleanupReservation,
};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use super::history::WorthQueryReleasedHistoryFollower;
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
    active: AtomicBool,
    _memory: Option<ExecutionMemoryReservation>,
}

#[derive(Clone)]
pub(crate) enum CleanupScope {
    Creation,
    Application,
    WorkspaceRetirement,
    ApplicationRetirement(WorthQueryRetiredProductOccurrence),
}

struct CleanupRegistryState {
    maximum: Option<usize>,
    next_identity: u64,
    reserved: usize,
    entries: ExecutionUniqueOrderedMap<u64, Arc<CleanupEntry>>,
}

#[derive(Clone)]
pub(crate) struct WorthQueryProductBranchOwnerCleanupRegistry {
    state: Arc<Mutex<CleanupRegistryState>>,
    follower: Arc<OnceLock<Weak<dyn WorthQueryReleasedHistoryFollower>>>,
}

impl WorthQueryProductBranchOwnerCleanupRegistry {
    pub(crate) fn new(maximum: Option<usize>) -> Self {
        Self {
            state: Arc::new(Mutex::new(CleanupRegistryState {
                maximum,
                next_identity: 1,
                reserved: 0,
                entries: ExecutionUniqueOrderedMap::default(),
            })),
            follower: Arc::new(OnceLock::new()),
        }
    }

    /// The one follower every history release these cleanups make reaches.
    pub(crate) fn follow_history_releases(
        &self,
        follower: Weak<dyn WorthQueryReleasedHistoryFollower>,
    ) {
        assert!(
            self.follower.set(follower).is_ok(),
            "one provider follows a product World's history releases"
        );
    }

    pub(super) fn history_released(&self) {
        if let Some(follower) = self.follower.get().and_then(Weak::upgrade) {
            follower.history_released();
        }
    }

    pub(super) fn reserve(
        &self,
        scope: CleanupScope,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    > {
        self.reserve_entry(scope, None, None)
            .map_err(|_| WorthQueryProductBranchOwnerCleanupCapacityExhausted)
    }

    fn reserve_entry(
        &self,
        scope: CleanupScope,
        memory: Option<ExecutionMemoryReservation>,
        request: Option<ExecutionRequest<'_, '_>>,
    ) -> Result<WorthQueryProductBranchOwnerCleanupReservation, WorthQueryCleanupReservationDenial>
    {
        let entry = Arc::new(CleanupEntry {
            record: Mutex::new(None),
            scope,
            active: AtomicBool::new(false),
            _memory: memory,
        });
        let mut state = lock(&self.state);
        if state
            .maximum
            .is_some_and(|maximum| state.entries.len() >= maximum)
        {
            return Err(WorthQueryCleanupReservationDenial::Capacity);
        }
        let identity = state.next_identity;
        let next = identity
            .checked_add(1)
            .ok_or(WorthQueryCleanupReservationDenial::Capacity)?;
        let reserved = state
            .reserved
            .checked_add(1)
            .ok_or(WorthQueryCleanupReservationDenial::Capacity)?;
        match request {
            Some(request) => state
                .entries
                .insert_cloned_native_for_request(&identity, &entry, request),
            None => state.entries.insert_cloned_native_in(
                &identity,
                &entry,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            ),
        }
        .map_err(WorthQueryCleanupReservationDenial::Storage)?;
        state.next_identity = next;
        state.reserved = reserved;
        Ok(WorthQueryProductBranchOwnerCleanupReservation::new(
            self.clone(),
            identity,
            entry,
        ))
    }

    pub(crate) fn available_capacity_for_test(&self) -> Option<usize> {
        let state = lock(&self.state);
        state
            .maximum
            .map(|maximum| maximum.saturating_sub(state.entries.len()))
    }

    pub(super) fn pending_identities(&self) -> Vec<u64> {
        let mut identities: Vec<_> = lock(&self.state)
            .entries
            .iter()
            .filter_map(|(id, entry)| entry.active.load(Ordering::Acquire).then_some(*id))
            .collect();
        identities.sort_unstable();
        identities
    }

    pub(super) fn pending_workspace_identities(&self) -> Vec<u64> {
        let mut identities: Vec<_> = lock(&self.state)
            .entries
            .iter()
            .filter_map(|(identity, entry)| {
                (entry.active.load(Ordering::Acquire)
                    && matches!(
                        entry.scope,
                        CleanupScope::Creation | CleanupScope::WorkspaceRetirement
                    ))
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
        let entry = lock(&self.state)
            .entries
            .get(&identity)
            .filter(|entry| entry.active.load(Ordering::Acquire))
            .cloned();
        let Some(entry) = entry else {
            return Vec::new();
        };

        lock(&entry.record)
            .as_ref()
            .map(WorthQueryProductBranchOwnerCleanupRecord::described_work)
            .unwrap_or_default()
    }

    pub(super) fn pending_signal_references(
        &self,
        identity: u64,
    ) -> Vec<worth_signal::facade::branch::ManagedSignalBranchReference> {
        let entry = lock(&self.state)
            .entries
            .get(&identity)
            .filter(|entry| entry.active.load(Ordering::Acquire))
            .cloned();
        let Some(entry) = entry else {
            return Vec::new();
        };

        lock(&entry.record)
            .as_ref()
            .map(|record| {
                record
                    .pending
                    .as_slice()
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
            .unwrap_or_default()
    }

    pub(super) fn release_history(
        &self,
        identity: u64,
        runtime: &WorthQueryProductRuntime,
    ) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial> {
        let entry = lock(&self.state)
            .entries
            .get(&identity)
            .filter(|entry| entry.active.load(Ordering::Acquire))
            .cloned()
            .ok_or(WorthQueryProductBranchOwnerCleanupDenial::CleanupUnavailable)?;
        let mut record = RetryRecordGuard::take(&entry.record)?;
        record.record_mut().release_retired_history(runtime)
    }

    pub(super) fn pending_application_retired_product_occurrences(
        &self,
    ) -> Vec<WorthQueryRetiredProductOccurrence> {
        let mut occurrences: Vec<_> = lock(&self.state)
            .entries
            .values()
            .filter(|entry| entry.active.load(Ordering::Acquire))
            .filter_map(|entry| match &entry.scope {
                CleanupScope::ApplicationRetirement(occurrence) => Some(occurrence.clone()),
                CleanupScope::Creation
                | CleanupScope::Application
                | CleanupScope::WorkspaceRetirement => None,
            })
            .collect();
        occurrences.sort_by(|left, right| {
            left.branch()
                .cmp(right.branch())
                .then_with(|| left.incarnation().cmp(&right.incarnation()))
        });
        occurrences
    }

    pub(super) fn retry(
        &self,
        identity: u64,
        runtime: &WorthQueryProductRuntime,
    ) -> Result<WorthQueryProductBranchOwnerCleanupReceipt, WorthQueryProductBranchOwnerCleanupDenial>
    {
        self.retry_admitted(identity, runtime, |_| Ok(()))
    }

    pub(super) fn retry_admitted(
        &self,
        identity: u64,
        runtime: &WorthQueryProductRuntime,
        checkpoint: impl FnMut(u64) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial>,
    ) -> Result<WorthQueryProductBranchOwnerCleanupReceipt, WorthQueryProductBranchOwnerCleanupDenial>
    {
        let entry = lock(&self.state)
            .entries
            .get(&identity)
            .filter(|entry| entry.active.load(Ordering::Acquire))
            .cloned()
            .ok_or(WorthQueryProductBranchOwnerCleanupDenial::CleanupUnavailable)?;
        let mut record = RetryRecordGuard::take(&entry.record)?;
        match record
            .record_mut()
            .retry(runtime, || self.after_component_progress(), checkpoint)
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

#[path = "registry/retry.rs"]
mod retry;
use retry::RetryRecordGuard;
#[cfg(test)]
#[path = "registry/testing.rs"]
mod testing;

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}

#[cfg(test)]
#[path = "registry/canonical_retirement.rs"]
mod canonical_retirement;
