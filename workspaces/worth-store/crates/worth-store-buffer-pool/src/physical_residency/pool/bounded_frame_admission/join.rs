use super::*;

enum BoundedJoinState {
    Loading,
    LoadFailed(PhysicalFrameLoadTerminal),
    Resident(RecordFrameCoordinate, Arc<Vec<u8>>),
}

impl PoolInner {
    pub(crate) fn join_bounded_loading(
        self: &Arc<Self>,
        key: PhysicalBoundedFrameKey,
        identity: PhysicalFrameLoadingIdentity,
    ) -> Result<PhysicalFrameLease, PhysicalFrameLoadTerminal> {
        let mut state = self.lock();
        loop {
            let posture = Self::observe_bounded_join_state(&state, key);
            match posture {
                Some((BoundedJoinState::Loading, Some(found), _)) if found == identity => {
                    #[cfg(test)]
                    self.bounded_join_waiters
                        .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                    state = self
                        .changed
                        .wait(state)
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    #[cfg(test)]
                    self.bounded_join_waiters
                        .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                }
                Some((BoundedJoinState::Resident(coordinate, bytes), Some(found), waiters))
                    if found == identity && waiters > 0 =>
                {
                    let entry = state
                        .frames
                        .get_bounded_mut(&key)
                        .expect("joined bounded frame remains indexed");
                    let BoundedFrameEntry::Resident { frame, .. } = entry else {
                        return Err(abandoned(identity));
                    };
                    frame.loading_waiters -= 1;
                    if frame.loading_waiters == 0 {
                        frame.loading_identity = None;
                    }
                    return Ok(PhysicalFrameLease {
                        owner: Arc::clone(self),
                        key: PhysicalFrameKey::new(self.store, coordinate),
                        bytes,
                        resident_generation: frame
                            .resident_generation
                            .expect("joined bounded frame has a byte-image generation"),
                    });
                }
                Some((BoundedJoinState::LoadFailed(terminal), Some(found), waiters))
                    if found == identity && waiters > 0 =>
                {
                    Self::release_failed_bounded_waiter(&mut state, key);
                    self.changed.notify_all();
                    return Err(terminal);
                }
                _ => return Err(abandoned(identity)),
            }
        }
    }

    fn observe_bounded_join_state(
        state: &PoolState,
        key: PhysicalBoundedFrameKey,
    ) -> Option<(BoundedJoinState, Option<PhysicalFrameLoadingIdentity>, u32)> {
        state
            .frames
            .get_bounded(&key)
            .and_then(|entry| match entry {
                BoundedFrameEntry::Loading {
                    identity, waiters, ..
                } => Some((BoundedJoinState::Loading, Some(*identity), *waiters)),
                BoundedFrameEntry::LoadFailed {
                    terminal, waiters, ..
                } => Some((
                    BoundedJoinState::LoadFailed(*terminal),
                    Some(terminal.identity()),
                    *waiters,
                )),
                BoundedFrameEntry::Resident { frame, .. } => {
                    let FrameState::Resident(bytes) = &frame.state else {
                        return None;
                    };
                    let coordinate = entry.resident_coordinate(key)?;
                    Some((
                        BoundedJoinState::Resident(coordinate, Arc::clone(bytes)),
                        frame.loading_identity,
                        frame.loading_waiters,
                    ))
                }
            })
    }

    pub(crate) fn release_bounded_loading_waiter(
        &self,
        key: PhysicalBoundedFrameKey,
        identity: PhysicalFrameLoadingIdentity,
    ) {
        let mut state = self.lock();
        let Some(posture) = Self::classify_bounded_waiter_release(&state, key, identity) else {
            return;
        };
        Self::release_bounded_waiter_posture(&mut state, key, posture);
        self.changed.notify_all();
    }

    fn classify_bounded_waiter_release(
        state: &PoolState,
        key: PhysicalBoundedFrameKey,
        identity: PhysicalFrameLoadingIdentity,
    ) -> Option<BoundedAccessPosture> {
        let entry = state.frames.get_bounded(&key)?;
        match entry {
            BoundedFrameEntry::Loading {
                identity: found,
                admitted_limit,
                waiters,
                ..
            } if *found == identity && *waiters > 0 => Some(BoundedAccessPosture::Loading {
                identity,
                admitted_limit: *admitted_limit,
            }),
            BoundedFrameEntry::LoadFailed {
                terminal, waiters, ..
            } if terminal.identity() == identity && *waiters > 0 => {
                Some(BoundedAccessPosture::LoadFailed(*terminal))
            }
            BoundedFrameEntry::Resident { frame, .. }
                if frame.loading_identity == Some(identity) && frame.loading_waiters > 0 =>
            {
                Some(BoundedAccessPosture::Resident(
                    entry
                        .resident_coordinate(key)
                        .expect("a bounded resident derives one exact coordinate"),
                ))
            }
            _ => None,
        }
    }

    fn release_bounded_waiter_posture(
        state: &mut PoolState,
        key: PhysicalBoundedFrameKey,
        posture: BoundedAccessPosture,
    ) {
        match posture {
            BoundedAccessPosture::Loading { .. } => {
                let entry = state
                    .frames
                    .get_bounded_mut(&key)
                    .expect("bounded waiter remains indexed");
                let BoundedFrameEntry::Loading { waiters, .. } = entry else {
                    return;
                };
                *waiters -= 1;
                state.accounting.unpin(false);
            }
            BoundedAccessPosture::Resident(coordinate) => {
                Self::release_resident_bounded_waiter(state, key, coordinate);
            }
            BoundedAccessPosture::LoadFailed(_)
            | BoundedAccessPosture::CandidatePublicationActive
            | BoundedAccessPosture::Absent => {
                Self::release_failed_bounded_waiter(state, key);
            }
        }
    }

    fn release_resident_bounded_waiter(
        state: &mut PoolState,
        key: PhysicalBoundedFrameKey,
        coordinate: RecordFrameCoordinate,
    ) {
        let became_unpinned = {
            let entry = state
                .frames
                .get_bounded_mut(&key)
                .expect("bounded waiter remains indexed");
            let BoundedFrameEntry::Resident { frame, .. } = entry else {
                return;
            };
            frame.loading_waiters -= 1;
            frame.pins -= 1;
            if frame.loading_waiters == 0 {
                frame.loading_identity = None;
            }
            frame.pins == 0
        };
        state.accounting.unpin(became_unpinned);
        if became_unpinned {
            state.append_evictable(coordinate);
            if state.closed {
                state.drain_all_legal_clean_frames();
            }
        }
    }

    fn release_failed_bounded_waiter(state: &mut PoolState, key: PhysicalBoundedFrameKey) {
        let (allocation_scope, remove_identity) = {
            let entry = state
                .frames
                .get_bounded_mut(&key)
                .expect("failed bounded waiter remains indexed");
            let BoundedFrameEntry::LoadFailed {
                allocation_scope,
                waiters,
                ..
            } = entry
            else {
                return;
            };
            *waiters -= 1;
            (*allocation_scope, *waiters == 0)
        };
        if remove_identity {
            state.frames.remove_bounded(&key);
            state
                .accounting
                .release_failed_loading_identity(allocation_scope);
        }
    }
}

fn abandoned(identity: PhysicalFrameLoadingIdentity) -> PhysicalFrameLoadTerminal {
    PhysicalFrameLoadTerminal::new(identity, PhysicalFrameLoadTerminalKind::FaultOwnerAbandoned)
}
