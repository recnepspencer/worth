use super::*;

impl PhysicalResidencyPool {
    pub fn open(
        store: StableStoreIdentity,
        limits: PhysicalResidencyLimits,
    ) -> Result<Self, PhysicalResidencyDenial> {
        let frame_count = limits.frame_entries() as usize;
        let minimum_metadata = frame_table::FrameTable::minimum_metadata_bytes(frame_count)
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<PoolState>()))
            .ok_or(PhysicalResidencyDenial::MetadataBudgetExceeded)?
            as u64;
        if minimum_metadata > limits.metadata_bytes() {
            return Err(PhysicalResidencyDenial::MetadataBudgetExceeded);
        }
        let frames = frame_table::FrameTable::open(frame_count)?;
        let metadata = frames
            .allocated_metadata_bytes()
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<PoolState>()))
            .ok_or(PhysicalResidencyDenial::MetadataBudgetExceeded)? as u64;
        if metadata > limits.metadata_bytes() {
            return Err(PhysicalResidencyDenial::MetadataBudgetExceeded);
        }
        let incarnation = PhysicalResidencyIncarnation::next()
            .ok_or(PhysicalResidencyDenial::AllocationFailed)?;
        let (allocation_recorder, allocation_events) =
            super::super::PhysicalResidencyAllocationEventRecorder::new(store, incarnation);
        Ok(Self {
            inner: Arc::new(PoolInner {
                store,
                incarnation,
                limits,
                allocation_events,
                state: Mutex::new(PoolState {
                    frames,
                    accounting: PhysicalResidencyAccounting::new(
                        minimum_metadata,
                        metadata,
                        allocation_recorder,
                    ),
                    evictable_head: None,
                    evictable_tail: None,
                    loading_frames: 0,
                    next_loading_ordinal: 1,
                    next_resident_generation: PhysicalResidentFrameGeneration::FIRST,
                    active_candidate_publications: 0,
                    dirty_generation: PhysicalDirtyGeneration::GENESIS,
                    accepting: true,
                    closed: false,
                }),
                changed: Condvar::new(),
                #[cfg(test)]
                bounded_join_waiters: std::sync::atomic::AtomicU32::new(0),
            }),
        })
    }

    pub fn store_identity(&self) -> StableStoreIdentity {
        self.inner.store
    }

    pub fn incarnation(&self) -> PhysicalResidencyIncarnation {
        self.inner.incarnation
    }

    pub fn allocation_events(&self) -> PhysicalResidencyAllocationEventObserver {
        self.inner.allocation_events.clone()
    }

    /// Invalidates clean validation when the owning Store lifecycle generation changes.
    pub fn invalidate_integrity_validation_for_runtime_transition(&self) {
        let mut state = self.inner.lock();
        state.invalidate_all_integrity_validation();
    }

    /// Verifies that `allocation` is live authority from this exact pool
    /// incarnation without reserving residency or exposing its scope.
    ///
    /// Adapters should call this before performing their own allocation or
    /// recording an attempted governed operation. Every pool admission still
    /// validates the grant again at the actual allocation boundary.
    pub fn validate_operation_allocation(
        &self,
        allocation: &OperationAllocationGrant,
    ) -> Result<(), PhysicalResidencyDenial> {
        self.allocation_scope(allocation).map(|_| ())
    }

    pub fn access_frame(
        &self,
        allocation: &OperationAllocationGrant,
        key: PhysicalFrameKey,
    ) -> Result<PhysicalFrameAccess, PhysicalResidencyDenial> {
        let scope = self
            .allocation_scope(allocation)
            .map_err(|reason| self.inner.record_denial(reason))?;
        self.inner
            .validate_key(key)
            .map_err(|reason| self.inner.record_denial(reason))?;
        self.inner.access_frame(scope, key)
    }

    pub fn access_bounded_frame(
        &self,
        allocation: &OperationAllocationGrant,
        key: PhysicalBoundedFrameKey,
    ) -> Result<PhysicalBoundedFrameAccess, PhysicalResidencyDenial> {
        let scope = self
            .allocation_scope(allocation)
            .map_err(|reason| self.inner.record_denial(reason))?;
        self.inner
            .validate_bounded_key(key)
            .map_err(|reason| self.inner.record_denial(reason))?;
        self.inner.access_bounded_frame(scope, key)
    }

    pub fn materialize_dirty_candidate<F>(
        &self,
        allocation: &ForegroundWriteAllocationGrant,
        key: PhysicalFrameKey,
        fill: F,
    ) -> Result<DirtyPhysicalFrame, PhysicalResidencyDenial>
    where
        F: FnOnce(&mut [u8]),
    {
        self.allocation_scope(allocation)?;
        self.inner
            .validate_key(key)
            .map_err(|reason| self.inner.record_denial(reason))?;
        let candidate = PhysicalCandidateFrameKey::fragment(key);
        let mut batch = self.reserve_candidate_frames(allocation, &[candidate])?;
        batch.reserve_next(candidate)?.materialize(fill)
    }

    pub fn reserve_candidate_frames<'grant>(
        &self,
        allocation: &'grant ForegroundWriteAllocationGrant,
        keys: &[PhysicalCandidateFrameKey],
    ) -> Result<PhysicalCandidateBatchReservation<'grant>, PhysicalResidencyDenial> {
        self.allocation_scope(allocation)?;
        let candidate_count = std::num::NonZeroUsize::new(keys.len()).ok_or_else(|| {
            self.inner
                .record_denial(PhysicalResidencyDenial::EmptyCandidateBatch)
        })?;
        self.begin_candidate_batch(allocation, candidate_count)?
            .reserve(keys)
    }

    pub fn begin_candidate_batch<'grant>(
        &self,
        allocation: &'grant ForegroundWriteAllocationGrant,
        candidate_count: std::num::NonZeroUsize,
    ) -> Result<PhysicalCandidateBatchAdmission<'grant>, PhysicalResidencyDenial> {
        self.allocation_scope(allocation)?;
        let allocation_bytes = Self::candidate_batch_operation_bytes(candidate_count)
            .ok_or_else(|| {
                self.inner
                    .record_denial(PhysicalResidencyDenial::AllocationFailed)
            })?
            .get();
        self.inner.validate_candidate_projection_start()?;
        let allocation_use = allocation.reserve_use(&self.inner, allocation_bytes)?;
        Ok(PhysicalCandidateBatchAdmission {
            owner: Arc::clone(&self.inner),
            allocation_use,
            candidate_count,
        })
    }

    pub fn candidate_batch_operation_bytes(
        candidate_count: std::num::NonZeroUsize,
    ) -> Option<std::num::NonZeroU64> {
        super::candidate_admission::candidate_batch_operation_bytes(candidate_count)
    }

    pub fn invalidate_clean(&self, key: PhysicalFrameKey) -> Result<(), PhysicalResidencyDenial> {
        self.inner.invalidate_clean(key)
    }

    pub fn begin_operation(
        &self,
        scope: PhysicalOperationAllocationScope,
        bytes: std::num::NonZeroU64,
    ) -> Result<OperationAllocationGrant, PhysicalResidencyDenial> {
        self.inner.reserve_operation(scope, bytes)?;
        Ok(OperationAllocationGrant {
            owner: Arc::clone(&self.inner),
            scope,
            bytes: bytes.get(),
            active_use_bytes: std::sync::atomic::AtomicU64::new(0),
        })
    }

    pub fn claim_writeback(
        &self,
        allocation: ForegroundWriteAllocationGrant,
        frames: &[PhysicalFrameKey],
    ) -> Result<PhysicalWritebackClaim, PhysicalResidencyDenial> {
        allocation.scope_for(&self.inner)?;
        let claimed = self.inner.claim_writeback(&allocation, frames)?;
        Ok(PhysicalWritebackClaim {
            owner: Arc::clone(&self.inner),
            writebehind: WriteBehindResidencyGrant::new(
                Arc::clone(&self.inner),
                allocation,
                claimed.frames,
            ),
            bytes: claimed.resident_bytes,
            range_postures: claimed.range_postures,
            armed: true,
        })
    }

    pub fn promote_clean_identity(
        &self,
        source: PhysicalFrameKey,
        target: PhysicalFrameKey,
    ) -> Result<(), PhysicalResidencyDenial> {
        self.inner.promote_clean_identity(source, target)
    }

    pub fn counters(&self) -> PhysicalResidencyCounters {
        self.inner.lock().accounting.snapshot()
    }

    pub fn close(&self) -> PhysicalResidencyShutdown {
        let mut state = self.inner.lock();
        if state.closed {
            return PhysicalResidencyShutdown::new(state.accounting.snapshot());
        }
        state.accepting = false;
        state.invalidate_all_integrity_validation();
        self.inner.changed.notify_all();
        state.drain_all_legal_clean_frames();
        state.closed = true;
        self.inner.changed.notify_all();
        PhysicalResidencyShutdown::new(state.accounting.snapshot())
    }

    pub fn drain_unpinned_clean_frames(&self) -> u64 {
        let mut state = self.inner.lock();
        if !state.accepting {
            return 0;
        }
        state.drain_all_legal_clean_frames()
    }

    fn allocation_scope(
        &self,
        allocation: &OperationAllocationGrant,
    ) -> Result<PhysicalOperationAllocationScope, PhysicalResidencyDenial> {
        allocation.scope_for(&self.inner)
    }
}
