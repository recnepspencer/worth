use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, RecordArtifactFile, RecordFrameCoordinate,
};

use crate::physical_runtime::{
    instance::{PhysicalExecutionCall, PhysicalStoreInstanceParts, PhysicalStoreWorkRuntime},
    record_serving::{CanonicalRecordMutationPort, CanonicalRecordReadPort},
    LifecycleGeneration, PhysicalWorkExecution, RuntimeIdentity,
};

use super::{
    super::{
        dirty::{
            AdmittedDirtyFrame, AdmittedPhysicalWriteback, PhysicalDirtyTransitionFailure,
            PhysicalWritebackExecution, PhysicalWritebackTransitionFailure,
            PreparedPhysicalWriteback, ReadyPhysicalWriteback,
        },
        frame_loading::CanonicalFrameReadSource,
        PhysicalPrefetchIntent, PhysicalPrefetchOutcome, PhysicalReadAheadIntent,
        PhysicalReadAheadOutcome, PhysicalResidencyWorkPort, PhysicalSpeculativeReadFailure,
    },
    CertificationFrameReadFailure, CertificationFrameWorkFailure, CertificationResidentFrame,
    CertificationScopeAdmissionFailure, CertificationScopedAllocation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CertificationResidencyBinding {
    store: StableStoreIdentity,
    runtime: RuntimeIdentity,
    generation: LifecycleGeneration,
}

impl CertificationResidencyBinding {
    pub(super) fn matches(
        self,
        store: StableStoreIdentity,
        runtime: RuntimeIdentity,
        generation: LifecycleGeneration,
    ) -> bool {
        self.store == store && self.runtime == runtime && self.generation == generation
    }
}

/// Certification-only observation and fault-driving access to physical
/// residency.
///
/// This capability exists to prove the ordinary Store composition boundary. It
/// cannot be constructed outside the serving runtime and is absent unless
/// certification authority is enabled.
#[derive(Clone)]
pub struct PhysicalResidencyCertification {
    binding: CertificationResidencyBinding,
    execution: PhysicalWorkExecution,
    residency: PhysicalResidencyWorkPort,
}

impl PhysicalResidencyCertification {
    pub(in crate::physical_runtime) fn from_parts(parts: &PhysicalStoreInstanceParts) -> Self {
        let generation = parts.core.lifecycle_generation();
        Self::for_generation(parts, generation)
    }

    pub(in crate::physical_runtime) fn stale_from_parts(
        parts: &PhysicalStoreInstanceParts,
    ) -> Self {
        let generation = parts
            .core
            .lifecycle_generation()
            .certification_predecessor();
        Self::for_generation(parts, generation)
    }

    fn for_generation(parts: &PhysicalStoreInstanceParts, generation: LifecycleGeneration) -> Self {
        let frame_ports = parts.residency.ports().clone();
        let mutation = CanonicalRecordMutationPort::new(
            &parts.work_runtime,
            generation,
            parts.work_admission,
            parts.scheduler_admission.clone(),
            std::sync::Arc::clone(&parts.record_work),
        );
        let writeback = mutation.frame_writeback_port(frame_ports.clone());
        Self {
            binding: CertificationResidencyBinding {
                store: parts
                    .work_runtime
                    .executor
                    .record_serving_media()
                    .store_identity(),
                runtime: parts.core.runtime_identity(),
                generation,
            },
            execution: PhysicalStoreWorkRuntime::execution(&parts.work_runtime, generation),
            residency: PhysicalResidencyWorkPort::new(
                frame_ports,
                CanonicalFrameReadSource::new(CanonicalRecordReadPort::new(
                    &parts.work_runtime,
                    generation,
                    parts.work_admission,
                    parts.scheduler_admission.clone(),
                    std::sync::Arc::clone(&parts.record_work),
                )),
                writeback,
                parts.core.lifecycle_state(),
            ),
        }
    }

    pub const fn lifecycle_generation(&self) -> LifecycleGeneration {
        self.binding.generation
    }

    pub fn pin_exact(
        &self,
        coordinate: RecordFrameCoordinate,
    ) -> Result<CertificationResidentFrame, CertificationFrameReadFailure> {
        let _call = self.admit_frame_call()?;
        let allocation = self
            .residency
            .begin_operation(
                worth_store_buffer_pool::PhysicalOperationAllocationScope::ForegroundRead,
                std::num::NonZeroU64::new(u64::from(coordinate.length()))
                    .expect("a physical frame coordinate has nonzero length"),
            )
            .map_err(CertificationFrameReadFailure::Residency)?;
        let frame = self
            .residency
            .load_exact(
                &allocation,
                coordinate,
                super::super::frame_loading::ExactFrameSourceExtent::CoordinateOnly,
            )
            .map_err(CertificationFrameReadFailure::from)?;
        Ok(CertificationResidentFrame::bind(
            self.binding,
            coordinate,
            frame,
        ))
    }

    pub fn pin_bounded(
        &self,
        artifact: RecordArtifactFile,
        limit: u32,
    ) -> Result<CertificationResidentFrame, CertificationFrameReadFailure> {
        let _call = self.admit_frame_call()?;
        let allocation = self
            .residency
            .begin_operation(
                worth_store_buffer_pool::PhysicalOperationAllocationScope::ForegroundRead,
                std::num::NonZeroU64::new(u64::from(limit))
                    .ok_or(CertificationFrameReadFailure::AccessLimitExceeded)?,
            )
            .map_err(CertificationFrameReadFailure::Residency)?;
        let frame = self
            .residency
            .load_bounded(&allocation, artifact, limit)
            .map_err(CertificationFrameReadFailure::from)?;
        let coordinate = RecordFrameCoordinate::new(artifact, 0, frame.len() as u32)
            .ok_or(CertificationFrameReadFailure::InvalidCoordinate)?;
        Ok(CertificationResidentFrame::bind(
            self.binding,
            coordinate,
            frame,
        ))
    }

    pub fn admit_operation_scope(
        &self,
        scope: crate::physical_runtime::PhysicalOperationAllocationScope,
        bytes: std::num::NonZeroU64,
    ) -> Result<CertificationScopedAllocation, CertificationScopeAdmissionFailure> {
        self.residency
            .begin_operation(scope, bytes)
            .map(CertificationScopedAllocation::bind)
            .map_err(CertificationScopeAdmissionFailure::from_denial)
    }

    pub fn prefetch(&self, intent: PhysicalPrefetchIntent) -> PhysicalPrefetchOutcome {
        let _call = match self.admit_frame_call() {
            Ok(call) => call,
            Err(failure) => {
                return PhysicalPrefetchOutcome::Failed(PhysicalSpeculativeReadFailure::Frame(
                    failure,
                ))
            }
        };
        self.residency.prefetch(intent, self.binding.generation)
    }

    pub fn read_ahead(&self, intent: PhysicalReadAheadIntent<'_>) -> PhysicalReadAheadOutcome {
        let _call = match self.admit_frame_call() {
            Ok(call) => call,
            Err(failure) => {
                return PhysicalReadAheadOutcome::FailedBeforeFrames(
                    PhysicalSpeculativeReadFailure::Frame(failure),
                )
            }
        };
        self.residency.read_ahead(intent, self.binding.generation)
    }

    pub fn counters(&self) -> worth_store_buffer_pool::PhysicalResidencyCounters {
        self.residency.counters()
    }

    pub fn allocation_trace(&self) -> super::super::PhysicalResidencyAllocationTrace {
        self.residency.allocation_trace()
    }

    pub fn drain_unpinned_clean_frames(&self) -> u64 {
        self.residency.drain_unpinned_clean_frames()
    }

    pub fn probe_competing_writeback_claim(
        &self,
        coordinate: RecordFrameCoordinate,
    ) -> Result<(), worth_store_buffer_pool::PhysicalResidencyDenial> {
        self.residency.probe_writeback_claim(coordinate)
    }

    pub fn admit_dirty_frame<F>(
        &self,
        lease: CertificationResidentFrame,
        fill: F,
    ) -> Result<AdmittedDirtyFrame, PhysicalDirtyTransitionFailure>
    where
        F: FnOnce(&[u8], &mut [u8]),
    {
        if !lease.belongs_to(
            self.binding.store,
            self.binding.runtime,
            self.binding.generation,
        ) {
            return Err(PhysicalDirtyTransitionFailure::StaleOrForeignFrame);
        }
        let coordinate = lease.coordinate();
        let allocation = self
            .residency
            .begin_foreground_write_operation(
                std::num::NonZeroU64::new(u64::from(coordinate.length()))
                    .expect("a physical frame coordinate has nonzero length"),
            )
            .map_err(PhysicalDirtyTransitionFailure::Residency)?;
        let (frame, source) = lease
            .into_dirty_candidate(&allocation, fill)
            .map_err(PhysicalDirtyTransitionFailure::Residency)?;
        Ok(AdmittedDirtyFrame::from_loaded_frame(
            coordinate, frame, source,
        ))
    }

    pub fn prepare_writeback(
        &self,
        dirty: AdmittedDirtyFrame,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> Result<PreparedPhysicalWriteback, PhysicalWritebackTransitionFailure> {
        self.residency.writeback().prepare(dirty, durability)
    }

    pub fn request_writeback(
        &self,
        prepared: PreparedPhysicalWriteback,
    ) -> Result<ReadyPhysicalWriteback, PhysicalWritebackTransitionFailure> {
        self.residency.writeback().request_ready(prepared)
    }

    pub fn bind_writeback_retry(
        &self,
        ready: crate::physical_runtime::ReadyPhysicalWork,
        dirty: AdmittedDirtyFrame,
    ) -> Result<ReadyPhysicalWriteback, PhysicalWritebackTransitionFailure> {
        self.residency.writeback().bind_retry_ready(ready, dirty)
    }

    pub fn admit_writeback(
        &self,
        ready: ReadyPhysicalWriteback,
    ) -> Result<AdmittedPhysicalWriteback, PhysicalWritebackTransitionFailure> {
        self.residency.writeback().admit(ready, None)
    }

    pub fn admit_writeback_retry(
        &self,
        ready: ReadyPhysicalWriteback,
        retry: crate::physical_runtime::PhysicalRetryCommand,
    ) -> Result<AdmittedPhysicalWriteback, PhysicalWritebackTransitionFailure> {
        self.residency.writeback().admit(ready, Some(retry))
    }

    pub fn execute_writeback(
        &self,
        admitted: AdmittedPhysicalWriteback,
    ) -> Result<PhysicalWritebackExecution, PhysicalWritebackTransitionFailure> {
        self.residency.writeback().execute(admitted)
    }

    fn admit_frame_call(&self) -> Result<PhysicalExecutionCall, CertificationFrameReadFailure> {
        self.execution.admit_call().map_err(|failure| {
            CertificationFrameReadFailure::PhysicalWork(CertificationFrameWorkFailure::PreEffect(
                failure,
            ))
        })
    }
}
