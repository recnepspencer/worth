use std::sync::Arc;

#[cfg(feature = "certification-test-authority")]
mod observation;
#[cfg(feature = "certification-test-authority")]
pub use observation::{FramePortCounterObserver, FramePortCounterSnapshot};

#[cfg(feature = "certification-test-authority")]
use worth_store_buffer_pool::{
    ForegroundReadAllocationGrant, PrefetchResidencyGrant, ReadAheadResidencyGrant,
};
use worth_store_buffer_pool::{
    ForegroundWriteAllocationGrant, FrameWritebackCleanAuthority, MaintenanceAllocationGrant,
    OperationAllocationGrant, PhysicalDirtyGenerationCaptureSession,
    PhysicalDirtyGenerationCaptureStep, PhysicalOperationAllocationScope,
    PhysicalResidencyCounters, PhysicalResidencyDenial, PhysicalResidencyLimits,
    PhysicalResidencyPool, PhysicalResidencyPoolOwner, PhysicalResidencyShutdown,
    PhysicalWritebackClaim,
};
use worth_store_physical_format::{store_namespace::StableStoreIdentity, RecordFrameCoordinate};

pub(in crate::physical_runtime::record_serving) use super::candidate_frame_residency::{
    CandidateFrame, CandidateFrameCoordinate, CandidateFrameDeclaration,
    CandidateFrameFailurePosture, CandidateFramePublicationPort, CandidateFrameRole,
    CandidateFrameSet, CandidateFrameWriteFailure, RecoverableCandidateFrameWriteFailure,
    StoreCandidateFramePublicationSession,
};
pub(in crate::physical_runtime::record_serving) use super::frame_loading::FrameLoadPort;

use super::candidate_frame_publishers::{
    BoundedCandidateFramePublisher, CandidateFrameCounterCells,
};
use super::frame_loading::BoundedFrameLoader;
use super::residency_observation::{
    PhysicalWritebackCounterCells, PhysicalWritebackCounterSnapshot,
};

#[derive(Clone)]
pub(in crate::physical_runtime) struct RecordFramePorts {
    pool: PhysicalResidencyPool,
    loader: BoundedFrameLoader,
    publisher: BoundedCandidateFramePublisher,
    writeback_clean: Arc<FrameWritebackCleanAuthority>,
    writeback_counters: Arc<PhysicalWritebackCounterCells>,
    resident_integrity_counters: Arc<crate::physical_runtime::ResidentAdmissionCounterCells>,
    #[cfg(feature = "certification-test-authority")]
    candidate_counters: Arc<CandidateFrameCounterCells>,
}

impl RecordFramePorts {
    pub(in crate::physical_runtime::record_serving) fn invalidate_released_arena_range(
        &self,
        range: worth_store_physical_format::ExtentArenaRange,
    ) -> Result<(), PhysicalResidencyDenial> {
        self.pool.invalidate_clean_range(
            worth_store_physical_format::RecordArtifactFile::ExtentArena {
                arena: range.arena().get(),
            },
            range.offset(),
            range.length(),
        )
    }

    pub(in crate::physical_runtime::record_serving) fn store_identity(
        &self,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.pool.store_identity()
    }

    pub(in crate::physical_runtime) fn bounded(
        store: StableStoreIdentity,
        limits: PhysicalResidencyLimits,
    ) -> Result<Self, PhysicalResidencyDenial> {
        let (pool, candidate_clean, writeback_clean) =
            PhysicalResidencyPoolOwner::open(store, limits)?.into_parts();
        let candidate_clean = Arc::new(candidate_clean);
        let candidate_counters = Arc::new(CandidateFrameCounterCells::default());
        let writeback_counters = Arc::new(PhysicalWritebackCounterCells::default());
        let resident_integrity_counters =
            Arc::new(crate::physical_runtime::ResidentAdmissionCounterCells::default());
        Ok(Self {
            loader: BoundedFrameLoader::new(pool.clone()),
            publisher: BoundedCandidateFramePublisher::new(
                pool.clone(),
                Arc::clone(&candidate_counters),
                candidate_clean,
            ),
            pool,
            writeback_clean: Arc::new(writeback_clean),
            writeback_counters,
            resident_integrity_counters,
            #[cfg(feature = "certification-test-authority")]
            candidate_counters,
        })
    }

    pub(in crate::physical_runtime::record_serving) const fn loader(
        &self,
    ) -> &(dyn FrameLoadPort + Send + Sync) {
        &self.loader
    }
    pub(in crate::physical_runtime::record_serving) const fn publisher(
        &self,
    ) -> &(dyn CandidateFramePublicationPort + Send + Sync) {
        &self.publisher
    }

    pub(in crate::physical_runtime) fn begin_operation(
        &self,
        scope: PhysicalOperationAllocationScope,
        bytes: std::num::NonZeroU64,
    ) -> Result<OperationAllocationGrant, PhysicalResidencyDenial> {
        self.pool.begin_operation(scope, bytes)
    }

    pub(in crate::physical_runtime) fn begin_foreground_write_operation(
        &self,
        bytes: std::num::NonZeroU64,
    ) -> Result<ForegroundWriteAllocationGrant, PhysicalResidencyDenial> {
        self.pool.begin_foreground_write_operation(bytes)
    }

    pub(in crate::physical_runtime) fn begin_checkpoint_capture(
        &self,
    ) -> Result<PhysicalDirtyGenerationCaptureSession, PhysicalResidencyDenial> {
        self.pool.begin_dirty_generation_capture()
    }

    pub(in crate::physical_runtime) fn checkpoint_capture_allocation(
        &self,
        bytes: std::num::NonZeroU64,
    ) -> Result<MaintenanceAllocationGrant, PhysicalResidencyDenial> {
        self.pool.begin_maintenance_operation(bytes)
    }

    pub(in crate::physical_runtime) fn capture_checkpoint_slice(
        &self,
        session: PhysicalDirtyGenerationCaptureSession,
        allocation: MaintenanceAllocationGrant,
    ) -> Result<PhysicalDirtyGenerationCaptureStep, PhysicalResidencyDenial> {
        self.pool
            .capture_next_dirty_generation_slice(session, allocation)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn begin_foreground_read_operation(
        &self,
        bytes: std::num::NonZeroU64,
    ) -> Result<ForegroundReadAllocationGrant, PhysicalResidencyDenial> {
        self.pool.begin_foreground_read_operation(bytes)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn admit_prefetch(
        &self,
        allocation: ForegroundReadAllocationGrant,
        coordinate: RecordFrameCoordinate,
    ) -> Result<PrefetchResidencyGrant, PhysicalResidencyDenial> {
        self.pool.admit_prefetch(allocation, coordinate)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn admit_read_ahead<'coordinates>(
        &self,
        allocation: ForegroundReadAllocationGrant,
        coordinates: &'coordinates [RecordFrameCoordinate],
    ) -> Result<ReadAheadResidencyGrant<'coordinates>, PhysicalResidencyDenial> {
        self.pool.admit_read_ahead(allocation, coordinates)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) const fn speculative_loader(
        &self,
    ) -> &BoundedFrameLoader {
        &self.loader
    }

    pub(in crate::physical_runtime) fn counters(&self) -> PhysicalResidencyCounters {
        self.pool.counters()
    }

    pub(in crate::physical_runtime) fn resident_integrity_counter_cells(
        &self,
    ) -> &crate::physical_runtime::ResidentAdmissionCounterCells {
        &self.resident_integrity_counters
    }

    pub(in crate::physical_runtime::record_serving) fn resident_integrity_counter_owner(
        &self,
    ) -> Arc<crate::physical_runtime::ResidentAdmissionCounterCells> {
        Arc::clone(&self.resident_integrity_counters)
    }

    pub(in crate::physical_runtime) fn resident_integrity_counters(
        &self,
    ) -> crate::physical_runtime::ResidentAdmissionCounters {
        self.resident_integrity_counters.snapshot()
    }

    pub(in crate::physical_runtime) fn invalidate_integrity_validation_for_runtime_transition(
        &self,
    ) {
        self.pool
            .invalidate_integrity_validation_for_runtime_transition();
    }

    pub(in crate::physical_runtime::record_serving) fn incarnation(
        &self,
    ) -> worth_store_buffer_pool::PhysicalResidencyIncarnation {
        self.pool.incarnation()
    }
    pub(in crate::physical_runtime) fn allocation_events(
        &self,
    ) -> worth_store_buffer_pool::PhysicalResidencyAllocationEventObserver {
        self.pool.allocation_events()
    }
    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn allocation_trace(
        &self,
    ) -> super::PhysicalResidencyAllocationTrace {
        super::PhysicalResidencyAllocationTrace::new(self.pool.allocation_events().trace())
    }
    pub(in crate::physical_runtime) fn writeback_counters(
        &self,
    ) -> PhysicalWritebackCounterSnapshot {
        self.writeback_counters.snapshot()
    }
    pub(in crate::physical_runtime) fn close(&self) -> PhysicalResidencyShutdown {
        self.pool.close()
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn drain_unpinned_clean_frames(&self) -> u64 {
        self.pool.drain_unpinned_clean_frames()
    }

    pub(in crate::physical_runtime) fn invalidate_completed_clean_frames(
        &self,
        coordinates: &[RecordFrameCoordinate],
    ) -> u64 {
        coordinates
            .iter()
            .filter(|coordinate| {
                self.pool
                    .invalidate_clean(worth_store_buffer_pool::PhysicalFrameKey::new(
                        self.pool.store_identity(),
                        **coordinate,
                    ))
                    .is_ok()
            })
            .count() as u64
    }

    pub(in crate::physical_runtime::record_serving) fn claim_writeback(
        &self,
        coordinate: RecordFrameCoordinate,
    ) -> Result<PhysicalWritebackClaim, PhysicalResidencyDenial> {
        let bytes = std::num::NonZeroU64::new(u64::from(coordinate.length()))
            .ok_or(PhysicalResidencyDenial::WriteBackExceedsDirtyPosture)?;
        let allocation = self.pool.begin_foreground_write_operation(bytes)?;
        self.pool.claim_writeback(
            allocation,
            &[worth_store_buffer_pool::PhysicalFrameKey::new(
                self.pool.store_identity(),
                coordinate,
            )],
        )
    }

    pub(in crate::physical_runtime::record_serving) fn writeback_clean_authority(
        &self,
    ) -> &FrameWritebackCleanAuthority {
        &self.writeback_clean
    }

    pub(in crate::physical_runtime::record_serving) fn observe_writeback_attempt(&self) {
        self.writeback_counters.observe_attempt();
    }

    pub(in crate::physical_runtime::record_serving) fn observe_exact_writeback_receipt(&self) {
        self.writeback_counters.observe_exact_receipt();
    }

    pub(in crate::physical_runtime::record_serving) fn observe_retryable_writeback(&self) {
        self.writeback_counters.observe_retryable();
    }

    pub(in crate::physical_runtime::record_serving) fn observe_writeback_inspection(
        &self,
        indeterminate: bool,
    ) {
        self.writeback_counters
            .observe_inspection_required(indeterminate);
    }

    pub(in crate::physical_runtime::record_serving) fn writeback_declaration(
        &self,
        claim: &PhysicalWritebackClaim,
        context: worth_store_buffer_pool::BufferPoolQueueDeclarationContext,
        durability: worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement,
    ) -> Result<
        worth_store_buffer_pool::BufferPoolWritebackQueueExecutionDeclaration,
        PhysicalResidencyDenial,
    > {
        let durability = match durability {
            worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite => {
                worth_store_buffer_pool::BufferPoolQueueWriteDurability::BufferedWrite
            }
            worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::
                FileDataSynchronization => {
                worth_store_buffer_pool::BufferPoolQueueWriteDurability::FileDataSynchronization
            }
        };
        worth_store_buffer_pool::BufferPoolWritebackQueueExecutionDeclaration::for_claim(
            claim, context, durability,
        )
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn observer(&self) -> FramePortCounterObserver {
        FramePortCounterObserver {
            pool: self.pool.clone(),
            candidate_counters: Arc::clone(&self.candidate_counters),
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn reject_next_candidate_publication(&self) {
        self.candidate_counters.reject_next_publication();
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn reject_next_candidate_retention(&self) {
        self.candidate_counters.reject_next_retention();
    }
}
