use super::*;

struct DirtyGenerationSliceBuffer {
    frames: Vec<PhysicalDirtyFrameBasis>,
    bytes: u64,
}

impl PhysicalResidencyPool {
    /// Reserves only a useful capture window within the current native pool
    /// envelope. The session and reservation remain bound to this exact pool.
    pub fn begin_dirty_generation_capture_allocation(
        &self,
        session: &PhysicalDirtyGenerationCaptureSession,
        maximum: std::num::NonZeroU64,
    ) -> Result<MaintenanceAllocationGrant, PhysicalResidencyDenial> {
        require_session_authority(&self.inner, session)?;
        let basis_bytes = super::super::dirty_generation::dirty_frame_basis_bytes() as u64;
        if maximum.get() < basis_bytes {
            return Err(self.inner.record_denial(
                PhysicalResidencyDenial::DirtyGenerationCaptureBudgetExceeded {
                    required: basis_bytes,
                    admitted: maximum.get(),
                },
            ));
        }
        let useful_bytes = u64::try_from(
            session
                .slot_limit()
                .saturating_sub(session.next_slot())
                .max(1),
        )
        .ok()
        .and_then(|slots| slots.checked_mul(basis_bytes))
        .ok_or_else(|| {
            self.inner
                .record_denial(PhysicalResidencyDenial::AllocationFailed)
        })?;
        let bytes = self
            .inner
            .reserve_dirty_capture_window(maximum.get().min(useful_bytes), basis_bytes)?;
        Ok(MaintenanceAllocationGrant::from_reserved(
            Arc::clone(&self.inner),
            bytes,
        ))
    }

    /// Fixes a dirty-generation frontier without materializing its frame set.
    pub fn begin_dirty_generation_capture(
        &self,
    ) -> Result<PhysicalDirtyGenerationCaptureSession, PhysicalResidencyDenial> {
        let (frontier, slot_limit) = self.inner.begin_dirty_generation_capture()?;
        Ok(PhysicalDirtyGenerationCaptureSession::new(
            self.inner.store,
            self.inner.incarnation,
            frontier,
            slot_limit,
        ))
    }

    /// Advances one capture session with memory bounded by a maintenance grant.
    pub fn capture_next_dirty_generation_slice(
        &self,
        session: PhysicalDirtyGenerationCaptureSession,
        allocation: MaintenanceAllocationGrant,
    ) -> Result<PhysicalDirtyGenerationCaptureStep, PhysicalResidencyDenial> {
        require_session_authority(&self.inner, &session)?;
        let allocation = allocation.into_operation();
        let scope = require_allocation_authority(&self.inner, &allocation)?;
        let mut buffer = allocate_slice_buffer(&self.inner, allocation.bytes())?;
        actualize_capture_allocation(&self.inner, scope, allocation.bytes(), buffer.bytes);
        let capacity = buffer.frames.capacity();
        let next_slot =
            self.inner
                .capture_dirty_source_slice(&session, &mut buffer.frames, capacity)?;
        let slice = PhysicalDirtyGenerationSlice::new(buffer.frames, buffer.bytes, allocation);
        Ok(PhysicalDirtyGenerationCaptureStep::from_advance(
            session, next_slot, slice,
        ))
    }
}

fn require_session_authority(
    owner: &PoolInner,
    session: &PhysicalDirtyGenerationCaptureSession,
) -> Result<(), PhysicalResidencyDenial> {
    if session.store_identity() == owner.store && session.pool_incarnation() == owner.incarnation {
        Ok(())
    } else {
        Err(owner.record_denial(PhysicalResidencyDenial::DirtyGenerationCaptureSessionMismatch))
    }
}

fn require_allocation_authority(
    owner: &Arc<PoolInner>,
    allocation: &OperationAllocationGrant,
) -> Result<PhysicalOperationAllocationScope, PhysicalResidencyDenial> {
    let scope = allocation
        .scope_for(owner)
        .map_err(|denial| owner.record_denial(denial))?;
    debug_assert_eq!(scope, PhysicalOperationAllocationScope::Maintenance);
    Ok(scope)
}

fn allocate_slice_buffer(
    owner: &PoolInner,
    admitted_bytes: u64,
) -> Result<DirtyGenerationSliceBuffer, PhysicalResidencyDenial> {
    let basis_bytes = super::super::dirty_generation::dirty_frame_basis_bytes() as u64;
    if admitted_bytes < basis_bytes {
        return Err(owner.record_denial(
            PhysicalResidencyDenial::DirtyGenerationCaptureBudgetExceeded {
                required: basis_bytes,
                admitted: admitted_bytes,
            },
        ));
    }
    let capacity = usize::try_from(admitted_bytes / basis_bytes)
        .map_err(|_| owner.record_denial(PhysicalResidencyDenial::AllocationFailed))?;
    let mut frames = Vec::new();
    frames
        .try_reserve_exact(capacity)
        .map_err(|_| owner.record_denial(PhysicalResidencyDenial::AllocationFailed))?;
    let bytes = u64::try_from(frames.capacity())
        .ok()
        .and_then(|capacity| capacity.checked_mul(basis_bytes))
        .ok_or_else(|| owner.record_denial(PhysicalResidencyDenial::AllocationFailed))?;
    if bytes > admitted_bytes {
        return Err(
            owner.record_denial(PhysicalResidencyDenial::AllocatorExceededReservation {
                requested: admitted_bytes,
                actual: bytes,
            }),
        );
    }
    Ok(DirtyGenerationSliceBuffer { frames, bytes })
}

fn actualize_capture_allocation(
    owner: &PoolInner,
    scope: PhysicalOperationAllocationScope,
    requested: u64,
    actual: u64,
) {
    for dimension in [
        PhysicalResidencyDimension::OperationBytes,
        PhysicalResidencyDimension::OperationScope(scope),
        PhysicalResidencyDimension::TotalBytes,
    ] {
        owner.actualize_allocation(PhysicalResidencyAllocationActualization::new(
            dimension,
            scope,
            PhysicalResidencyRequestedAllocationUnits::new(requested),
            PhysicalResidencyActualAllocationUnits::new(actual),
        ));
    }
}

impl PoolInner {
    fn begin_dirty_generation_capture(
        &self,
    ) -> Result<(PhysicalDirtyGeneration, usize), PhysicalResidencyDenial> {
        let mut state = self.lock();
        if !state.accepting {
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        Ok((state.dirty_generation, state.frames.slot_count()))
    }

    fn capture_dirty_source_slice(
        &self,
        session: &PhysicalDirtyGenerationCaptureSession,
        frames: &mut Vec<PhysicalDirtyFrameBasis>,
        capacity: usize,
    ) -> Result<usize, PhysicalResidencyDenial> {
        let mut state = self.lock();
        if !state.accepting {
            return Err(Self::deny(&mut state, PhysicalResidencyDenial::PoolClosed));
        }
        let mut next_slot = session.next_slot();
        for (index, coordinate, entry) in state.frames.resident_entries_from(next_slot) {
            next_slot = index + 1;
            if entry.dirty {
                let generation = entry
                    .dirty_generation
                    .expect("every resident dirty frame has one dirty generation");
                if generation <= session.frontier() {
                    frames.push(PhysicalDirtyFrameBasis::new(
                        PhysicalFrameKey::new(self.store, coordinate),
                        generation,
                    ));
                    if frames.len() == capacity {
                        break;
                    }
                }
            }
        }
        if frames.len() < capacity {
            next_slot = session.slot_limit();
        }
        Ok(next_slot)
    }
}

impl PoolState {
    pub(super) fn advance_dirty_generation(
        &mut self,
    ) -> Result<PhysicalDirtyGeneration, PhysicalResidencyDenial> {
        let next = self
            .dirty_generation
            .successor()
            .ok_or(PhysicalResidencyDenial::DirtyGenerationExhausted)?;
        self.dirty_generation = next;
        Ok(next)
    }
}

#[cfg(test)]
impl PhysicalResidencyPool {
    pub(in crate::physical_residency) fn force_dirty_generation_frontier(&self, generation: u64) {
        self.inner.lock().dirty_generation = PhysicalDirtyGeneration::for_test(generation);
    }
}
