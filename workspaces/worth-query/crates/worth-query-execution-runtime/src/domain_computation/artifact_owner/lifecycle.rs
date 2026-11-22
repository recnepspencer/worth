use std::ops::{Deref, DerefMut};
use std::sync::{MutexGuard, RwLockReadGuard};

use super::{
    WorthQueryArtifactDenial, WorthQueryArtifactDenialKind, WorthQueryArtifactDisposition,
    WorthQueryArtifactHandleGuard, WorthQueryArtifactOwnerSnapshot,
    WorthQueryArtifactProviderReleasePosture, WorthQueryRuntimeArtifactLifecycle,
    WorthQueryRuntimeArtifactOwner,
};

impl WorthQueryRuntimeArtifactOwner {
    pub(super) fn snapshot(&self) -> WorthQueryArtifactOwnerSnapshot {
        let _snapshot = self.snapshot_gate.lifecycle_mutation();
        self.lifecycle.snapshot()
    }

    pub(super) fn validate_guard(
        &self,
        guard: WorthQueryArtifactHandleGuard,
    ) -> Result<(), WorthQueryArtifactDenial> {
        let mut state = self.lifecycle();
        validate_guard(self, &mut state, guard)
    }

    pub(super) fn validate_borrow_generation(
        &self,
        generation: u64,
    ) -> Result<(), WorthQueryArtifactDenial> {
        let mut state = self.lifecycle();
        state.counters.lifecycle_generation_checks += 1;
        ensure_live(self, &state)?;
        if !state.active_borrows.contains(&generation) {
            return Err(self.denial(
                WorthQueryArtifactDenialKind::StaleLifecycleGeneration,
                "artifact view carries an inactive borrow generation",
            ));
        }
        Ok(())
    }

    pub(super) fn admit_transfer(&self, generation: u64) -> Result<u64, WorthQueryArtifactDenial> {
        use worth_query_installation::facade::WorthQueryArtifactMovePosture;

        if self.binding().contract.contract().carriage().movement()
            != WorthQueryArtifactMovePosture::Required
        {
            return Err(self.denial(
                WorthQueryArtifactDenialKind::MovementForbidden,
                "installed artifact contract forbids ownership movement",
            ));
        }
        let mut state = self.lifecycle();
        validate_guard(
            self,
            &mut state,
            WorthQueryArtifactHandleGuard::Owner(generation),
        )?;
        if !state.active_borrows.is_empty() {
            return Err(self.denial(
                WorthQueryArtifactDenialKind::ActiveBorrow,
                "artifact cannot transfer while a borrow is active",
            ));
        }
        state.owner_generation += 1;
        state.disposition = WorthQueryArtifactDisposition::Transferred;
        state.counters.transfer_admissions += 1;
        Ok(state.owner_generation)
    }

    pub(super) fn admit_borrow(
        &self,
        guard: WorthQueryArtifactHandleGuard,
    ) -> Result<u64, WorthQueryArtifactDenial> {
        use worth_query_installation::facade::WorthQueryArtifactBorrowPosture;

        if self.binding().contract.contract().carriage().borrowing()
            != WorthQueryArtifactBorrowPosture::SharedReadOnly
        {
            return Err(self.denial(
                WorthQueryArtifactDenialKind::BorrowForbidden,
                "installed artifact contract forbids shared borrowing",
            ));
        }
        let mut state = self.lifecycle();
        validate_guard(self, &mut state, guard)?;
        state.next_borrow_generation += 1;
        let generation = state.next_borrow_generation;
        let inserted = state.active_borrows.insert(generation);
        debug_assert!(inserted);
        state.disposition = WorthQueryArtifactDisposition::Borrowed;
        state.counters.borrow_admissions += 1;
        Ok(generation)
    }

    pub(super) fn release_borrow(&self, generation: u64) -> Result<(), WorthQueryArtifactDenial> {
        let should_dispose = {
            let mut state = self.lifecycle();
            if !state.active_borrows.remove(&generation) {
                return Err(self.denial(
                    WorthQueryArtifactDenialKind::StaleLifecycleGeneration,
                    "artifact borrow generation is no longer active",
                ));
            }
            if state.active_borrows.is_empty() && !state.close_requested {
                state.disposition = WorthQueryArtifactDisposition::Released;
            }
            mark_disposed_if_unowned(&mut state)
        };
        self.dispose_provider_if_required(should_dispose);
        Ok(())
    }

    pub(super) fn admit_lease(&self, generation: u64) -> Result<u64, WorthQueryArtifactDenial> {
        use worth_query_installation::facade::{
            WorthQueryArtifactBorrowPosture, WorthQueryArtifactClonePosture,
        };

        let contract = self.binding().contract.contract();
        let lease_is_declared = contract.lifecycle().is_reusable()
            && (contract.carriage().borrowing() == WorthQueryArtifactBorrowPosture::SharedReadOnly
                || matches!(
                    contract.carriage().clone_posture(),
                    WorthQueryArtifactClonePosture::Declared { .. }
                ));
        if !lease_is_declared {
            return Err(self.denial(
                WorthQueryArtifactDenialKind::LeaseForbidden,
                "installed artifact contract does not admit a retained lease",
            ));
        }
        let mut state = self.lifecycle();
        validate_guard(
            self,
            &mut state,
            WorthQueryArtifactHandleGuard::Owner(generation),
        )?;
        state.next_lease_generation += 1;
        let generation = state.next_lease_generation;
        let inserted = state.active_leases.insert(generation);
        debug_assert!(inserted);
        state.disposition = WorthQueryArtifactDisposition::Leased;
        state.counters.lease_admissions += 1;
        Ok(generation)
    }

    pub(super) fn release_lease(
        &self,
        generation: u64,
        disposition: WorthQueryArtifactDisposition,
    ) -> Result<WorthQueryArtifactProviderReleasePosture, WorthQueryArtifactDenial> {
        let should_dispose = {
            let mut state = self.lifecycle();
            if !state.active_leases.remove(&generation) {
                return Err(self.denial(
                    WorthQueryArtifactDenialKind::StaleLifecycleGeneration,
                    "artifact lease generation is no longer active",
                ));
            }
            state.disposition = disposition;
            mark_disposed_if_unowned(&mut state)
        };
        self.dispose_provider_if_required(should_dispose);
        Ok(self.snapshot().provider_release())
    }

    pub(super) fn release_owner(
        &self,
        generation: u64,
        disposition: WorthQueryArtifactDisposition,
        require_no_lease: bool,
    ) -> Result<WorthQueryArtifactProviderReleasePosture, WorthQueryArtifactDenial> {
        let should_dispose = {
            let mut state = self.lifecycle();
            validate_guard(
                self,
                &mut state,
                WorthQueryArtifactHandleGuard::Owner(generation),
            )?;
            if !state.active_borrows.is_empty() {
                return Err(self.denial(
                    WorthQueryArtifactDenialKind::ActiveBorrow,
                    "artifact cannot be disposed while a borrow is active",
                ));
            }
            if require_no_lease && !state.active_leases.is_empty() {
                return Err(self.denial(
                    WorthQueryArtifactDenialKind::ActiveLease,
                    "artifact cannot be explicitly disposed while a lease is active",
                ));
            }
            state.owner_active = false;
            state.owner_generation += 1;
            state.disposition = disposition;
            mark_disposed_if_unowned(&mut state)
        };
        self.dispose_provider_if_required(should_dispose);
        Ok(self.snapshot().provider_release())
    }

    pub(super) fn release_guard_on_drop(
        &self,
        guard: WorthQueryArtifactHandleGuard,
        disposition: WorthQueryArtifactDisposition,
    ) {
        let should_dispose = {
            let mut state = self.lifecycle();
            if state.disposed {
                return;
            }
            match guard {
                WorthQueryArtifactHandleGuard::Owner(generation)
                    if state.owner_active && state.owner_generation == generation =>
                {
                    state.owner_active = false;
                    state.owner_generation += 1;
                }
                WorthQueryArtifactHandleGuard::Lease(generation)
                    if state.active_leases.remove(&generation) => {}
                _ => return,
            }
            if !state.close_requested {
                state.disposition = disposition;
            }
            mark_disposed_if_unowned(&mut state)
        };
        self.dispose_provider_if_required(should_dispose);
    }

    pub(super) fn request_registry_close(&self, disposition: WorthQueryArtifactDisposition) {
        let should_dispose = {
            let mut state = self.lifecycle();
            if state.disposed || state.close_requested {
                return;
            }
            state.close_requested = true;
            state.owner_active = false;
            state.owner_generation += 1;
            state.active_leases.clear();
            state.disposition = disposition;
            mark_disposed_if_unowned(&mut state)
        };
        self.dispose_provider_if_required(should_dispose);
    }

    fn lifecycle(&self) -> WorthQueryArtifactLifecycleMutation<'_> {
        WorthQueryArtifactLifecycleMutation {
            _snapshot: self.snapshot_gate.lifecycle_mutation(),
            state: self.lifecycle.lock(),
        }
    }
}

struct WorthQueryArtifactLifecycleMutation<'a> {
    _snapshot: RwLockReadGuard<'a, ()>,
    state: MutexGuard<'a, WorthQueryRuntimeArtifactLifecycle>,
}

impl Deref for WorthQueryArtifactLifecycleMutation<'_> {
    type Target = WorthQueryRuntimeArtifactLifecycle;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for WorthQueryArtifactLifecycleMutation<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

fn validate_guard(
    owner: &WorthQueryRuntimeArtifactOwner,
    state: &mut WorthQueryRuntimeArtifactLifecycle,
    guard: WorthQueryArtifactHandleGuard,
) -> Result<(), WorthQueryArtifactDenial> {
    state.counters.lifecycle_generation_checks += 1;
    ensure_live(owner, state)?;
    let active = match guard {
        WorthQueryArtifactHandleGuard::Owner(generation) => {
            state.owner_active && state.owner_generation == generation
        }
        WorthQueryArtifactHandleGuard::Lease(generation) => {
            state.active_leases.contains(&generation)
        }
    };
    if !active {
        return Err(owner.denial(
            WorthQueryArtifactDenialKind::StaleLifecycleGeneration,
            "artifact handle carries an inactive lifecycle generation",
        ));
    }
    Ok(())
}

fn ensure_live(
    owner: &WorthQueryRuntimeArtifactOwner,
    state: &WorthQueryRuntimeArtifactLifecycle,
) -> Result<(), WorthQueryArtifactDenial> {
    if state.disposed || state.close_requested {
        Err(owner.denial(
            WorthQueryArtifactDenialKind::AlreadyDisposed,
            "artifact owner is disposed or closed by its workflow run",
        ))
    } else {
        Ok(())
    }
}

fn mark_disposed_if_unowned(state: &mut WorthQueryRuntimeArtifactLifecycle) -> bool {
    if !state.owner_active
        && state.active_borrows.is_empty()
        && state.active_leases.is_empty()
        && !state.disposed
    {
        state.disposed = true;
        state.disposition = WorthQueryArtifactDisposition::Disposed;
        state.provider_release = WorthQueryArtifactProviderReleasePosture::Pending;
        true
    } else {
        false
    }
}
