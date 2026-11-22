use std::collections::BTreeSet;
use std::sync::{Mutex, MutexGuard};

use super::{
    WorthQueryArtifactDisposition, WorthQueryArtifactLifecycleCounters,
    WorthQueryArtifactOwnerSnapshot, WorthQueryArtifactProviderReleaseEvidence,
    WorthQueryArtifactProviderReleasePosture,
};

pub(super) struct WorthQueryArtifactLifecycleRecord {
    state: Mutex<WorthQueryRuntimeArtifactLifecycle>,
}

impl WorthQueryArtifactLifecycleRecord {
    pub(super) fn new(retained_bytes: usize) -> Self {
        Self {
            state: Mutex::new(WorthQueryRuntimeArtifactLifecycle::new(retained_bytes)),
        }
    }

    pub(super) fn lock(&self) -> MutexGuard<'_, WorthQueryRuntimeArtifactLifecycle> {
        self.state
            .lock()
            .expect("artifact lifecycle lock must remain available")
    }

    pub(super) fn snapshot(&self) -> WorthQueryArtifactOwnerSnapshot {
        let state = self.lock();
        WorthQueryArtifactOwnerSnapshot::new(
            usize::from(state.owner_active),
            state.active_borrows.len(),
            state.active_leases.len(),
            state.owner_generation,
            state.disposed,
            state.provider_release,
            state.counters,
        )
    }

    pub(super) fn record_provider_release(
        &self,
        evidence: WorthQueryArtifactProviderReleaseEvidence,
    ) {
        let mut state = self.lock();
        debug_assert_eq!(
            state.provider_release,
            WorthQueryArtifactProviderReleasePosture::Pending
        );
        state.provider_release = WorthQueryArtifactProviderReleasePosture::from_evidence(evidence);
        state.counters.provider_disposals = state.counters.provider_disposals.saturating_add(1);
        state.counters.provider_destructor_attempts = state
            .counters
            .provider_destructor_attempts
            .saturating_add(1);
        if evidence.recovery_required() {
            state.counters.provider_release_failures =
                state.counters.provider_release_failures.saturating_add(1);
        }
    }
}

pub(super) struct WorthQueryRuntimeArtifactLifecycle {
    pub(super) owner_generation: u64,
    pub(super) owner_active: bool,
    pub(super) next_borrow_generation: u64,
    pub(super) active_borrows: BTreeSet<u64>,
    pub(super) next_lease_generation: u64,
    pub(super) active_leases: BTreeSet<u64>,
    pub(super) close_requested: bool,
    pub(super) disposed: bool,
    pub(super) disposition: WorthQueryArtifactDisposition,
    pub(super) provider_release: WorthQueryArtifactProviderReleasePosture,
    pub(super) counters: WorthQueryArtifactLifecycleCounters,
}

impl WorthQueryRuntimeArtifactLifecycle {
    fn new(retained_bytes: usize) -> Self {
        Self {
            owner_generation: 1,
            owner_active: true,
            next_borrow_generation: 0,
            active_borrows: BTreeSet::new(),
            next_lease_generation: 0,
            active_leases: BTreeSet::new(),
            close_requested: false,
            disposed: false,
            disposition: WorthQueryArtifactDisposition::Produced,
            provider_release: WorthQueryArtifactProviderReleasePosture::Retained,
            counters: WorthQueryArtifactLifecycleCounters {
                production_admissions: 1,
                owner_registrations: 1,
                retained_bytes,
                peak_retained_bytes: retained_bytes,
                ..WorthQueryArtifactLifecycleCounters::default()
            },
        }
    }
}
