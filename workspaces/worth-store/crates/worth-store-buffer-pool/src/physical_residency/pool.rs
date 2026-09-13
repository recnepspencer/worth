use super::{
    integrity_validation::{
        invalidate_clean_frame_validation, CleanFrameIntegrityValidationRecord,
        PhysicalResidentFrameGeneration,
    },
    DirtyPhysicalFrame, ForegroundWriteAllocationGrant, MaintenanceAllocationGrant,
    OperationAllocationGrant, PhysicalBoundedFrameAccess, PhysicalBoundedFrameFaultOwner,
    PhysicalBoundedFrameFaultWaiter, PhysicalCandidateBatchAdmission,
    PhysicalCandidateBatchReservation, PhysicalCandidateFrameReservation, PhysicalDirtyFrameBasis,
    PhysicalDirtyGeneration, PhysicalDirtyGenerationCaptureSession,
    PhysicalDirtyGenerationCaptureStep, PhysicalDirtyGenerationSlice, PhysicalFrameAccess,
    PhysicalFrameFaultOwner, PhysicalFrameFaultWaiter, PhysicalFrameLease,
    PhysicalFrameLoadTerminal, PhysicalFrameLoadTerminalKind, PhysicalFrameLoadingIdentity,
    PhysicalFrameRemoval, PhysicalOperationAllocationScope, PhysicalResidencyAccounting,
    PhysicalResidencyActualAllocationUnits, PhysicalResidencyAllocationActualization,
    PhysicalResidencyAllocationEventObserver, PhysicalResidencyCounters, PhysicalResidencyDenial,
    PhysicalResidencyDimension, PhysicalResidencyLimits, PhysicalResidencyPressureDemand,
    PhysicalResidencyPressureDenial, PhysicalResidencyRequestedAllocationUnits,
    PhysicalResidencyShutdown, PhysicalWritebackClaim, WriteBehindResidencyGrant,
};
use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex, MutexGuard},
};
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, RecordArtifactFile, RecordFrameCoordinate,
};

mod bounded_frame_admission;
mod candidate_admission;
mod dirty_generation_capture;
mod dirty_transition;
mod eviction;
mod frame_admission;
mod frame_table;
mod identity_transition;
mod integrity_validation;
mod operation_accounting;
mod pin_lifecycle;
mod public_api;
mod writeback_claim;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalFrameKey {
    store: StableStoreIdentity,
    coordinate: RecordFrameCoordinate,
}

impl PhysicalFrameKey {
    pub const fn new(store: StableStoreIdentity, coordinate: RecordFrameCoordinate) -> Self {
        Self { store, coordinate }
    }
    pub const fn store(self) -> StableStoreIdentity {
        self.store
    }
    pub const fn coordinate(self) -> RecordFrameCoordinate {
        self.coordinate
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalCandidateFrameKey {
    frame: PhysicalFrameKey,
    coverage: PhysicalCandidateFrameCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PhysicalCandidateFrameCoverage {
    Fragment,
    CompleteArtifact,
}

impl PhysicalCandidateFrameKey {
    pub const fn fragment(frame: PhysicalFrameKey) -> Self {
        Self {
            frame,
            coverage: PhysicalCandidateFrameCoverage::Fragment,
        }
    }

    pub const fn complete_artifact(frame: PhysicalFrameKey) -> Option<Self> {
        if frame.coordinate().offset() != 0 {
            return None;
        }
        Some(Self {
            frame,
            coverage: PhysicalCandidateFrameCoverage::CompleteArtifact,
        })
    }

    pub const fn frame_key(self) -> PhysicalFrameKey {
        self.frame
    }

    const fn is_complete_artifact(self) -> bool {
        matches!(
            self.coverage,
            PhysicalCandidateFrameCoverage::CompleteArtifact
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalBoundedFrameKey {
    store: StableStoreIdentity,
    artifact: RecordArtifactFile,
    limit: std::num::NonZeroU32,
}

impl PhysicalBoundedFrameKey {
    pub const fn new(
        store: StableStoreIdentity,
        artifact: RecordArtifactFile,
        limit: std::num::NonZeroU32,
    ) -> Self {
        Self {
            store,
            artifact,
            limit,
        }
    }

    pub const fn store(self) -> StableStoreIdentity {
        self.store
    }

    pub const fn artifact(self) -> RecordArtifactFile {
        self.artifact
    }

    pub const fn limit(self) -> u32 {
        self.limit.get()
    }
}

#[derive(Debug, Clone)]
pub struct PhysicalResidencyPool {
    pub(crate) inner: Arc<PoolInner>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalResidencyIncarnation(u64);

impl PhysicalResidencyIncarnation {
    fn next() -> Option<Self> {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        NEXT.fetch_update(
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
            |current| current.checked_add(1),
        )
        .ok()
        .map(Self)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
pub(crate) struct PoolInner {
    store: StableStoreIdentity,
    incarnation: PhysicalResidencyIncarnation,
    limits: PhysicalResidencyLimits,
    allocation_events: PhysicalResidencyAllocationEventObserver,
    state: Mutex<PoolState>,
    changed: Condvar,
    #[cfg(test)]
    bounded_join_waiters: std::sync::atomic::AtomicU32,
}

#[derive(Debug)]
struct PoolState {
    frames: frame_table::FrameTable,
    accounting: PhysicalResidencyAccounting,
    evictable_head: Option<RecordFrameCoordinate>,
    evictable_tail: Option<RecordFrameCoordinate>,
    loading_frames: u32,
    next_loading_ordinal: u64,
    next_resident_generation: PhysicalResidentFrameGeneration,
    active_candidate_publications: u32,
    dirty_generation: PhysicalDirtyGeneration,
    accepting: bool,
    closed: bool,
}

#[derive(Debug)]
struct FrameEntry {
    state: FrameState,
    origin: FrameOrigin,
    allocation_scope: PhysicalOperationAllocationScope,
    pins: u32,
    dirty: bool,
    dirty_generation: Option<PhysicalDirtyGeneration>,
    writeback_claimed: bool,
    bytes: u64,
    older_evictable: Option<RecordFrameCoordinate>,
    newer_evictable: Option<RecordFrameCoordinate>,
    loading_identity: Option<PhysicalFrameLoadingIdentity>,
    loading_waiters: u32,
    artifact_posture: FrameArtifactPosture,
    resident_generation: Option<PhysicalResidentFrameGeneration>,
    integrity_validation: Option<CleanFrameIntegrityValidationRecord>,
}

impl FrameEntry {
    fn accounting_removal(&self) -> PhysicalFrameRemoval {
        PhysicalFrameRemoval::new(self.allocation_scope, self.bytes, self.pins)
            .with_dirty(self.dirty)
            .with_candidate(self.origin.is_candidate())
    }

    fn invalidate_integrity_validation(&mut self) {
        invalidate_clean_frame_validation(&mut self.integrity_validation);
    }
}

#[derive(Debug)]
enum FrameState {
    Loading,
    LoadFailed(PhysicalFrameLoadTerminal),
    CandidateReserved,
    Resident(Arc<Vec<u8>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameOrigin {
    Fault,
    Candidate,
    DirtyReplacement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameArtifactPosture {
    Fragment,
    CompleteCandidate,
    CompleteResident,
}

impl FrameOrigin {
    const fn is_candidate(self) -> bool {
        matches!(self, Self::Candidate | Self::DirtyReplacement)
    }

    const fn writeback_range_posture(
        self,
        coordinate: RecordFrameCoordinate,
    ) -> crate::PhysicalWritebackRangePosture {
        if matches!(self, Self::Candidate) && coordinate.offset() > 0 {
            crate::PhysicalWritebackRangePosture::CandidateArtifactTail
        } else {
            crate::PhysicalWritebackRangePosture::ExistingRange
        }
    }
}

impl PoolInner {
    pub(crate) const fn store_identity(&self) -> StableStoreIdentity {
        self.store
    }

    pub(crate) const fn incarnation(&self) -> PhysicalResidencyIncarnation {
        self.incarnation
    }

    #[cfg(test)]
    pub(crate) fn bounded_join_waiters(&self) -> u32 {
        self.bounded_join_waiters
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub(crate) fn counters(&self) -> PhysicalResidencyCounters {
        self.lock().accounting.snapshot()
    }

    fn lock(&self) -> MutexGuard<'_, PoolState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn validate_key(
        &self,
        key: PhysicalFrameKey,
    ) -> Result<(), PhysicalResidencyDenial> {
        if key.store != self.store {
            return Err(PhysicalResidencyDenial::WrongStore);
        }
        if key.coordinate.length() as u64 > self.limits.resident_bytes() {
            return Err(PhysicalResidencyDenial::FrameLargerThanResidentBudget);
        }
        Ok(())
    }

    fn validate_bounded_key(
        &self,
        key: PhysicalBoundedFrameKey,
    ) -> Result<(), PhysicalResidencyDenial> {
        if key.store != self.store {
            return Err(PhysicalResidencyDenial::WrongStore);
        }
        if u64::from(key.limit()) > self.limits.resident_bytes() {
            return Err(PhysicalResidencyDenial::FrameLargerThanResidentBudget);
        }
        Ok(())
    }

    pub(crate) fn record_source_load(&self) {
        self.lock().accounting.record_source_load();
    }

    pub(crate) fn actualize_allocation(
        &self,
        actualization: PhysicalResidencyAllocationActualization,
    ) {
        self.lock().accounting.actualize_allocation(actualization);
    }

    pub(crate) fn record_denial(&self, denial: PhysicalResidencyDenial) -> PhysicalResidencyDenial {
        Self::deny(&mut self.lock(), denial)
    }

    fn current_admitted_bytes(&self, state: &PoolState) -> u64 {
        state.accounting.admitted_bytes()
    }

    fn pressure(
        &self,
        state: &mut PoolState,
        demand: PhysicalResidencyPressureDemand,
    ) -> PhysicalResidencyDenial {
        state
            .accounting
            .deny_dimension(demand.dimension, demand.scope, demand.requested);
        PhysicalResidencyDenial::Pressure(PhysicalResidencyPressureDenial::new(
            self.store,
            self.incarnation,
            demand,
        ))
    }

    fn deny(state: &mut PoolState, denial: PhysicalResidencyDenial) -> PhysicalResidencyDenial {
        state.accounting.deny();
        denial
    }
}

impl PhysicalCandidateFrameReservation {
    pub(crate) fn new(
        owner: Arc<PoolInner>,
        candidate: PhysicalCandidateFrameKey,
        scope: PhysicalOperationAllocationScope,
    ) -> Self {
        Self {
            owner,
            candidate,
            scope,
            armed: true,
        }
    }
}
