use super::*;

mod clean_publication;
mod frame_space;
mod loading_waiters;

#[derive(Clone, Copy)]
enum FrameAccessPosture {
    Loading(PhysicalFrameLoadingIdentity),
    LoadFailed(PhysicalFrameLoadTerminal),
    CandidateReserved,
    Resident,
    Absent,
}

impl PoolInner {
    pub(super) fn access_frame(
        self: &Arc<Self>,
        scope: PhysicalOperationAllocationScope,
        key: PhysicalFrameKey,
    ) -> Result<PhysicalFrameAccess, PhysicalResidencyDenial> {
        let mut state = self.lock();
        loop {
            if !state.accepting {
                return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
            }
            match Self::frame_access_posture(&state, key) {
                FrameAccessPosture::Loading(identity) => {
                    return self.attach_loading_waiter(&mut state, scope, key, identity);
                }
                FrameAccessPosture::LoadFailed(terminal) => {
                    return Err(Self::deny(
                        &mut state,
                        PhysicalResidencyDenial::FrameLoadTerminated(terminal),
                    ));
                }
                FrameAccessPosture::CandidateReserved => {
                    state = self
                        .changed
                        .wait(state)
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                }
                FrameAccessPosture::Resident => {
                    return self
                        .pin_resident_frame(&mut state, scope, key)
                        .map(PhysicalFrameAccess::Hit);
                }
                FrameAccessPosture::Absent => {
                    return self
                        .reserve_loading(&mut state, scope, key)
                        .map(PhysicalFrameAccess::Fault);
                }
            }
        }
    }

    fn frame_access_posture(state: &PoolState, key: PhysicalFrameKey) -> FrameAccessPosture {
        let Some(entry) = state.frames.get(&key.coordinate) else {
            return FrameAccessPosture::Absent;
        };
        match &entry.state {
            FrameState::Loading => FrameAccessPosture::Loading(
                entry
                    .loading_identity
                    .expect("a loading frame has one loading identity"),
            ),
            FrameState::LoadFailed(terminal) => FrameAccessPosture::LoadFailed(*terminal),
            FrameState::CandidateReserved => FrameAccessPosture::CandidateReserved,
            FrameState::Resident(_) => FrameAccessPosture::Resident,
        }
    }

    fn reserve_loading(
        self: &Arc<Self>,
        state: &mut PoolState,
        scope: PhysicalOperationAllocationScope,
        key: PhysicalFrameKey,
    ) -> Result<PhysicalFrameFaultOwner, PhysicalResidencyDenial> {
        let ordinal = state.next_loading_ordinal;
        let next_ordinal = ordinal
            .checked_add(1)
            .ok_or_else(|| Self::deny(state, PhysicalResidencyDenial::AllocationFailed))?;
        self.reserve_frame_space(state, scope, key.coordinate.length() as u64)?;
        state.next_loading_ordinal = next_ordinal;
        let identity = PhysicalFrameLoadingIdentity::new(self.incarnation, ordinal);
        state.frames.insert(
            key.coordinate,
            FrameEntry {
                state: FrameState::Loading,
                origin: FrameOrigin::Fault,
                allocation_scope: scope,
                pins: 1,
                dirty: false,
                dirty_generation: None,
                writeback_claimed: false,
                bytes: key.coordinate.length() as u64,
                older_evictable: None,
                newer_evictable: None,
                loading_identity: Some(identity),
                loading_waiters: 0,
                artifact_posture: FrameArtifactPosture::Fragment,
                resident_generation: None,
                integrity_validation: None,
            },
        );
        state
            .accounting
            .admit_frame(scope, u64::from(key.coordinate.length()), false, false);
        state.loading_frames += 1;
        Ok(PhysicalFrameFaultOwner {
            owner: Arc::clone(self),
            key,
            identity,
            scope,
            armed: true,
        })
    }

    pub(crate) fn finish_loading(
        self: &Arc<Self>,
        key: PhysicalFrameKey,
        identity: PhysicalFrameLoadingIdentity,
        bytes: Arc<Vec<u8>>,
    ) -> Result<PhysicalFrameLease, (PhysicalResidencyDenial, PhysicalFrameLoadTerminal)> {
        let mut state = self.lock();
        if !state.accepting {
            let terminal = Self::fail_loading_state(
                &mut state,
                key,
                identity,
                PhysicalFrameLoadTerminalKind::PoolClosed,
            );
            let denial = Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed);
            self.changed.notify_all();
            return Err((denial, terminal));
        }
        let matches_identity = state.frames.get(&key.coordinate).is_some_and(|entry| {
            matches!(entry.state, FrameState::Loading) && entry.loading_identity == Some(identity)
        });
        if !matches_identity {
            let terminal = PhysicalFrameLoadTerminal::new(
                identity,
                PhysicalFrameLoadTerminalKind::FaultOwnerAbandoned,
            );
            let denial = Self::deny(
                &mut state,
                PhysicalResidencyDenial::FrameLoadTerminated(terminal),
            );
            return Err((denial, terminal));
        }
        let resident_generation = state.advance_resident_generation().map_err(|denial| {
            let terminal = Self::fail_loading_state(
                &mut state,
                key,
                identity,
                PhysicalFrameLoadTerminalKind::AllocationFailed,
            );
            (Self::deny(&mut state, denial), terminal)
        })?;
        let entry = state
            .frames
            .get_mut(&key.coordinate)
            .expect("loading reservation exists");
        entry.state = FrameState::Resident(Arc::clone(&bytes));
        entry.resident_generation = Some(resident_generation);
        entry.invalidate_integrity_validation();
        state.loading_frames -= 1;
        state.accounting.finish_loading();
        self.changed.notify_all();
        Ok(PhysicalFrameLease {
            owner: Arc::clone(self),
            key,
            bytes,
            resident_generation,
        })
    }

    pub(crate) fn fail_loading(
        &self,
        key: PhysicalFrameKey,
        identity: PhysicalFrameLoadingIdentity,
        kind: PhysicalFrameLoadTerminalKind,
    ) -> PhysicalFrameLoadTerminal {
        let mut state = self.lock();
        let terminal = Self::fail_loading_state(&mut state, key, identity, kind);
        self.changed.notify_all();
        terminal
    }

    pub(crate) fn abandon_loading(
        &self,
        key: PhysicalFrameKey,
        identity: PhysicalFrameLoadingIdentity,
    ) {
        self.fail_loading(
            key,
            identity,
            PhysicalFrameLoadTerminalKind::FaultOwnerAbandoned,
        );
    }

    fn fail_loading_state(
        state: &mut PoolState,
        key: PhysicalFrameKey,
        identity: PhysicalFrameLoadingIdentity,
        kind: PhysicalFrameLoadTerminalKind,
    ) -> PhysicalFrameLoadTerminal {
        let terminal = PhysicalFrameLoadTerminal::new(identity, kind);
        let Some(entry) = state.frames.get(&key.coordinate) else {
            return terminal;
        };
        if !matches!(entry.state, FrameState::Loading) || entry.loading_identity != Some(identity) {
            return terminal;
        }
        let bytes = entry.bytes;
        let pins = entry.pins;
        let waiters = entry.loading_waiters;
        let allocation_scope = entry.allocation_scope;
        state.loading_frames -= 1;
        state
            .accounting
            .fail_loading(allocation_scope, bytes, pins, waiters > 0);
        if waiters == 0 {
            state.frames.remove(&key.coordinate);
        } else {
            let entry = state
                .frames
                .get_mut(&key.coordinate)
                .expect("retained failed loading identity remains indexed");
            entry.state = FrameState::LoadFailed(terminal);
            entry.pins = 0;
        }
        terminal
    }
}
