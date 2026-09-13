use super::*;

impl PoolInner {
    pub(super) fn pin_resident_frame(
        self: &Arc<Self>,
        state: &mut PoolState,
        scope: PhysicalOperationAllocationScope,
        key: PhysicalFrameKey,
    ) -> Result<PhysicalFrameLease, PhysicalResidencyDenial> {
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
            .get(&key.coordinate)
            .expect("a classified resident frame remains indexed");
        let bytes = match &entry.state {
            FrameState::Resident(bytes) => Arc::clone(bytes),
            _ => {
                return Err(Self::deny(state, PhysicalResidencyDenial::FrameNotResident));
            }
        };
        let resident_generation = entry
            .resident_generation
            .expect("a resident frame has an installed byte-image generation");
        let was_unpinned = entry.pins == 0;
        if was_unpinned {
            state.detach_evictable(key.coordinate);
            if state.accounting.pinned_frames() >= self.limits.pinned_frames() {
                state.append_evictable(key.coordinate);
                let current = u64::from(state.accounting.pinned_frames());
                return Err(self.pressure(
                    state,
                    PhysicalResidencyPressureDemand {
                        dimension: PhysicalResidencyDimension::PinnedFrames,
                        scope,
                        requested: 1,
                        current,
                        limit: u64::from(self.limits.pinned_frames()),
                    },
                ));
            }
        }
        state
            .frames
            .get_mut(&key.coordinate)
            .expect("classified resident frame remains indexed")
            .pins += 1;
        state.accounting.pin(was_unpinned);
        Ok(PhysicalFrameLease {
            owner: Arc::clone(self),
            key,
            bytes,
            resident_generation,
        })
    }

    pub(crate) fn release_pin(&self, key: PhysicalFrameKey) {
        let mut state = self.lock();
        let Some(entry) = state.frames.get_mut(&key.coordinate) else {
            return;
        };
        if entry.pins == 0 {
            return;
        }
        entry.pins -= 1;
        let became_unpinned = entry.pins == 0;
        let became_evictable =
            became_unpinned && !entry.dirty && matches!(entry.state, FrameState::Resident(_));
        state.accounting.unpin(became_unpinned);
        if became_evictable {
            state.append_evictable(key.coordinate);
            if state.closed {
                state.drain_all_legal_clean_frames();
            }
        }
        self.changed.notify_all();
    }
}
