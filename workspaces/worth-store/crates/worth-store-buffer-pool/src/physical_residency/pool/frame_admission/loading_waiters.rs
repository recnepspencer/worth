use super::*;

enum LoadingJoinState {
    Loading,
    Failed(PhysicalFrameLoadTerminal),
    Resident(Arc<Vec<u8>>),
    Other,
}

impl FrameState {
    fn clone_for_loading_join(&self) -> LoadingJoinState {
        match self {
            Self::Loading => LoadingJoinState::Loading,
            Self::LoadFailed(terminal) => LoadingJoinState::Failed(*terminal),
            Self::Resident(bytes) => LoadingJoinState::Resident(Arc::clone(bytes)),
            Self::CandidateReserved => LoadingJoinState::Other,
        }
    }
}

impl PoolInner {
    pub(super) fn attach_loading_waiter(
        self: &Arc<Self>,
        state: &mut PoolState,
        scope: PhysicalOperationAllocationScope,
        key: PhysicalFrameKey,
        identity: PhysicalFrameLoadingIdentity,
    ) -> Result<PhysicalFrameAccess, PhysicalResidencyDenial> {
        if state.accounting.pin_leases() >= self.limits.pin_leases() {
            let current = u64::from(state.accounting.pin_leases());
            return Err(self.pressure(
                state,
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::PinLeases,
                    scope,
                    requested: 1,
                    current,
                    limit: u64::from(self.limits.pin_leases()),
                },
            ));
        }
        let entry = state
            .frames
            .get_mut(&key.coordinate)
            .expect("the classified loading frame remains indexed");
        if entry.loading_identity != Some(identity) || !matches!(entry.state, FrameState::Loading) {
            return Err(Self::deny(
                state,
                PhysicalResidencyDenial::FrameLoadTerminated(PhysicalFrameLoadTerminal::new(
                    identity,
                    PhysicalFrameLoadTerminalKind::FaultOwnerAbandoned,
                )),
            ));
        }
        entry.pins += 1;
        entry.loading_waiters += 1;
        state.accounting.attach_loading_waiter();
        Ok(PhysicalFrameAccess::Coalesced(PhysicalFrameFaultWaiter {
            owner: Arc::clone(self),
            key,
            identity,
            armed: true,
        }))
    }

    pub(crate) fn join_loading(
        self: &Arc<Self>,
        key: PhysicalFrameKey,
        identity: PhysicalFrameLoadingIdentity,
    ) -> Result<PhysicalFrameLease, PhysicalFrameLoadTerminal> {
        let mut state = self.lock();
        loop {
            let posture = state.frames.get(&key.coordinate).map(|entry| {
                (
                    entry.state.clone_for_loading_join(),
                    entry.loading_identity,
                    entry.loading_waiters,
                )
            });
            match posture {
                Some((LoadingJoinState::Loading, Some(found), _)) if found == identity => {
                    state = self
                        .changed
                        .wait(state)
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                }
                Some((LoadingJoinState::Resident(bytes), Some(found), waiters))
                    if found == identity && waiters > 0 =>
                {
                    let entry = state
                        .frames
                        .get_mut(&key.coordinate)
                        .expect("joined resident frame remains indexed");
                    entry.loading_waiters -= 1;
                    if entry.loading_waiters == 0 {
                        entry.loading_identity = None;
                    }
                    return Ok(PhysicalFrameLease {
                        owner: Arc::clone(self),
                        key,
                        bytes,
                        resident_generation: entry
                            .resident_generation
                            .expect("joined resident frame has a byte-image generation"),
                    });
                }
                Some((LoadingJoinState::Failed(terminal), Some(found), waiters))
                    if found == identity && waiters > 0 =>
                {
                    Self::release_failed_waiter(&mut state, key);
                    self.changed.notify_all();
                    return Err(terminal);
                }
                _ => {
                    return Err(PhysicalFrameLoadTerminal::new(
                        identity,
                        PhysicalFrameLoadTerminalKind::FaultOwnerAbandoned,
                    ));
                }
            }
        }
    }

    pub(crate) fn release_loading_waiter(
        &self,
        key: PhysicalFrameKey,
        identity: PhysicalFrameLoadingIdentity,
    ) {
        let mut state = self.lock();
        let Some(entry) = state.frames.get(&key.coordinate) else {
            return;
        };
        if entry.loading_identity != Some(identity) || entry.loading_waiters == 0 {
            return;
        }
        match &entry.state {
            FrameState::LoadFailed(_) => Self::release_failed_waiter(&mut state, key),
            FrameState::Loading => {
                let entry = state
                    .frames
                    .get_mut(&key.coordinate)
                    .expect("loading waiter frame remains indexed");
                entry.loading_waiters -= 1;
                entry.pins -= 1;
                state.accounting.unpin(false);
            }
            FrameState::Resident(_) => {
                let (became_unpinned, became_evictable) = {
                    let entry = state
                        .frames
                        .get_mut(&key.coordinate)
                        .expect("resident waiter frame remains indexed");
                    entry.loading_waiters -= 1;
                    entry.pins -= 1;
                    let became_unpinned = entry.pins == 0;
                    if entry.loading_waiters == 0 {
                        entry.loading_identity = None;
                    }
                    (
                        became_unpinned,
                        became_unpinned && !entry.dirty && !entry.writeback_claimed,
                    )
                };
                state.accounting.unpin(became_unpinned);
                if became_evictable {
                    state.append_evictable(key.coordinate);
                    if state.closed {
                        state.drain_all_legal_clean_frames();
                    }
                }
            }
            FrameState::CandidateReserved => {}
        }
        self.changed.notify_all();
    }

    fn release_failed_waiter(state: &mut PoolState, key: PhysicalFrameKey) {
        let (allocation_scope, remove_identity) = {
            let entry = state
                .frames
                .get_mut(&key.coordinate)
                .expect("failed loading waiter frame remains indexed");
            entry.loading_waiters -= 1;
            (entry.allocation_scope, entry.loading_waiters == 0)
        };
        if remove_identity {
            state.frames.remove(&key.coordinate);
            state
                .accounting
                .release_failed_loading_identity(allocation_scope);
        }
    }
}
