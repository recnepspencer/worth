use std::{num::NonZeroU64, sync::Arc};

use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::{
    candidate_frame_residency::{CandidateFrameSet, StoreCandidateFramePublicationSession},
    frame_loading::{
        CanonicalFrameReadSource, ExactFrameSourceExtent, FrameLoadFailure, LoadedPhysicalFrame,
    },
    frame_ports::RecordFramePorts,
    FrameWritebackPort,
};

/// The single Store-private capability for ordinary serving-frame residency.
///
/// It keeps the pool port and canonical physical-work source inseparable, so
/// an ordinary reader cannot retain the pool while substituting a direct media
/// source or retain the source while bypassing residency admission.
#[derive(Clone)]
pub(in crate::physical_runtime::record_serving) struct PhysicalResidencyWorkPort {
    access: Arc<PhysicalResidencyWorkAccess>,
    source: CanonicalFrameReadSource,
}

struct PhysicalResidencyWorkAccess {
    frame_ports: RecordFramePorts,
    writeback: FrameWritebackPort,
    lifecycle: Arc<crate::physical_runtime::lifecycle::LifecycleState>,
}

const _: () =
    assert!(std::mem::size_of::<PhysicalResidencyWorkPort>() <= std::mem::size_of::<usize>() * 4);

impl PhysicalResidencyWorkPort {
    pub(in crate::physical_runtime::record_serving) fn store_identity(
        &self,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.access.frame_ports.store_identity()
    }

    pub(in crate::physical_runtime::record_serving) fn new(
        frame_ports: RecordFramePorts,
        source: CanonicalFrameReadSource,
        writeback: FrameWritebackPort,
        lifecycle: Arc<crate::physical_runtime::lifecycle::LifecycleState>,
    ) -> Self {
        Self {
            access: Arc::new(PhysicalResidencyWorkAccess {
                frame_ports,
                writeback,
                lifecycle,
            }),
            source,
        }
    }

    pub(in crate::physical_runtime::record_serving) fn resident_admission_context(
        &self,
    ) -> crate::physical_runtime::integrity::resident_admission::load::ResidentAdmissionContext<
        'static,
    > {
        crate::physical_runtime::integrity::resident_admission::load::ResidentAdmissionContext::from_shared(
            Arc::clone(&self.access.lifecycle),
            self.access
                .frame_ports
                .resident_integrity_counter_owner(),
        )
    }

    pub(in crate::physical_runtime::record_serving) fn for_scan(mut self) -> Self {
        self.source = self.source.for_scan();
        self
    }

    pub(in crate::physical_runtime::record_serving) fn begin_operation(
        &self,
        scope: worth_store_buffer_pool::PhysicalOperationAllocationScope,
        bytes: NonZeroU64,
    ) -> Result<
        worth_store_buffer_pool::OperationAllocationGrant,
        worth_store_buffer_pool::PhysicalResidencyDenial,
    > {
        self.access.frame_ports.begin_operation(scope, bytes)
    }

    pub(in crate::physical_runtime::record_serving) fn begin_foreground_write_operation(
        &self,
        bytes: NonZeroU64,
    ) -> Result<
        worth_store_buffer_pool::ForegroundWriteAllocationGrant,
        worth_store_buffer_pool::PhysicalResidencyDenial,
    > {
        self.access
            .frame_ports
            .begin_foreground_write_operation(bytes)
    }

    pub(in crate::physical_runtime::record_serving) fn load_exact(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        coordinate: RecordFrameCoordinate,
        source_extent: ExactFrameSourceExtent,
    ) -> Result<LoadedPhysicalFrame, FrameLoadFailure> {
        self.access.frame_ports.loader().load_exact(
            allocation,
            &self.source,
            coordinate.artifact(),
            coordinate.offset(),
            coordinate.length(),
            source_extent,
        )
    }

    pub(in crate::physical_runtime::record_serving) fn load_bounded(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        artifact: RecordArtifactFile,
        limit: u32,
    ) -> Result<LoadedPhysicalFrame, FrameLoadFailure> {
        self.access
            .frame_ports
            .loader()
            .load_bounded(allocation, &self.source, artifact, limit)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn admit_prefetch(
        &self,
        coordinate: RecordFrameCoordinate,
    ) -> Result<
        worth_store_buffer_pool::PrefetchResidencyGrant,
        worth_store_buffer_pool::PhysicalResidencyDenial,
    > {
        let bytes = NonZeroU64::new(u64::from(coordinate.length()))
            .expect("a physical frame coordinate has nonzero length");
        let allocation = self
            .access
            .frame_ports
            .begin_foreground_read_operation(bytes)?;
        self.access
            .frame_ports
            .admit_prefetch(allocation, coordinate)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn admit_read_ahead<'coordinates>(
        &self,
        bytes: NonZeroU64,
        coordinates: &'coordinates [RecordFrameCoordinate],
    ) -> Result<
        worth_store_buffer_pool::ReadAheadResidencyGrant<'coordinates>,
        worth_store_buffer_pool::PhysicalResidencyDenial,
    > {
        let allocation = self
            .access
            .frame_ports
            .begin_foreground_read_operation(bytes)?;
        self.access
            .frame_ports
            .admit_read_ahead(allocation, coordinates)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn load_prefetch(
        &self,
        grant: &worth_store_buffer_pool::PrefetchResidencyGrant,
    ) -> Result<LoadedPhysicalFrame, FrameLoadFailure> {
        self.access
            .frame_ports
            .speculative_loader()
            .load_prefetch(grant, &self.source)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn load_read_ahead(
        &self,
        grant: &worth_store_buffer_pool::ReadAheadFrameGrant<'_, '_>,
    ) -> Result<LoadedPhysicalFrame, FrameLoadFailure> {
        self.access
            .frame_ports
            .speculative_loader()
            .load_read_ahead(grant, &self.source)
    }

    pub(in crate::physical_runtime::record_serving) fn begin_candidate_publication<'allocation>(
        &self,
        allocation: &'allocation worth_store_buffer_pool::ForegroundWriteAllocationGrant,
        declaration: CandidateFrameSet,
    ) -> Result<StoreCandidateFramePublicationSession<'allocation>, super::super::RecordAppendDenial>
    {
        StoreCandidateFramePublicationSession::begin(
            self.access.frame_ports.publisher(),
            allocation,
            declaration,
        )
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn counters(
        &self,
    ) -> worth_store_buffer_pool::PhysicalResidencyCounters {
        self.access.frame_ports.counters()
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn allocation_trace(
        &self,
    ) -> super::PhysicalResidencyAllocationTrace {
        self.access.frame_ports.allocation_trace()
    }

    pub(in crate::physical_runtime::record_serving) fn writeback(&self) -> &FrameWritebackPort {
        &self.access.writeback
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn drain_unpinned_clean_frames(&self) -> u64 {
        self.access.frame_ports.drain_unpinned_clean_frames()
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn probe_writeback_claim(
        &self,
        coordinate: RecordFrameCoordinate,
    ) -> Result<(), worth_store_buffer_pool::PhysicalResidencyDenial> {
        self.access
            .frame_ports
            .claim_writeback(coordinate)
            .map(drop)
    }
}
