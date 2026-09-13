#[cfg(feature = "certification-test-authority")]
use worth_store_buffer_pool::{
    DirtyPhysicalFrame, ForegroundWriteAllocationGrant, PhysicalDirtyReplacementError,
    PhysicalResidencyDenial,
};
use worth_store_buffer_pool::{PhysicalFrameKey, PhysicalFrameLease};
use worth_store_physical_format::RecordFrameCoordinate;

use super::{FrameLoadFailure, FrameLoadFailureKind};
use crate::physical_runtime::record_serving::residency::frame_work_trace::FrameWorkTrace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) enum PhysicalFrameAccessOrigin {
    Hit,
    Coalesced,
    Fault,
}

pub(in crate::physical_runtime::record_serving) struct LoadedPhysicalFrame {
    lease: PhysicalFrameLease,
    #[cfg(feature = "certification-test-authority")]
    origin: PhysicalFrameAccessOrigin,
    work: FrameWorkTrace,
    projection_failure:
        Option<crate::physical_runtime::instance::PhysicalProjectionFailureCapability>,
}

impl LoadedPhysicalFrame {
    pub(super) fn bind(
        store: worth_store_physical_format::store_namespace::StableStoreIdentity,
        coordinate: RecordFrameCoordinate,
        lease: PhysicalFrameLease,
        _origin: PhysicalFrameAccessOrigin,
        work: FrameWorkTrace,
        projection_failure: Option<
            crate::physical_runtime::instance::PhysicalProjectionFailureCapability,
        >,
    ) -> Result<Self, FrameLoadFailure> {
        if lease.key() != PhysicalFrameKey::new(store, coordinate) {
            if let Some(projection_failure) = projection_failure {
                projection_failure.consume();
            }
            return Err(
                FrameLoadFailure::new(FrameLoadFailureKind::ReturnedFrameIdentityMismatch)
                    .with_complete_work_trace(work),
            );
        }
        Ok(Self {
            lease,
            #[cfg(feature = "certification-test-authority")]
            origin: _origin,
            work,
            projection_failure,
        })
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) const fn origin(
        &self,
    ) -> PhysicalFrameAccessOrigin {
        self.origin
    }

    pub(in crate::physical_runtime::record_serving) const fn work_trace(&self) -> FrameWorkTrace {
        self.work
    }

    pub(in crate::physical_runtime::record_serving) const fn coordinate(
        &self,
    ) -> RecordFrameCoordinate {
        self.lease.key().coordinate()
    }

    pub(in crate::physical_runtime::record_serving) const fn lease(&self) -> &PhysicalFrameLease {
        &self.lease
    }

    pub(in crate::physical_runtime::record_serving) fn reject_projection_failure(mut self) {
        if let Some(projection_failure) = self.projection_failure.take() {
            projection_failure.consume();
        }
    }

    pub(in crate::physical_runtime::record_serving) fn copy_range_into(
        &self,
        range: std::ops::Range<usize>,
        target: &mut [u8],
    ) {
        self.lease.copy_range_into(range, target);
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn fill_dirty_candidate<F>(
        self,
        allocation: &ForegroundWriteAllocationGrant,
        fill: F,
    ) -> Result<(DirtyPhysicalFrame, FrameWorkTrace), PhysicalResidencyDenial>
    where
        F: FnOnce(&[u8], &mut [u8]),
    {
        let work = self.work;
        self.lease
            .begin_dirty_replacement(allocation)?
            .replace(|source, target| {
                fill(source, target);
                Ok::<_, std::convert::Infallible>(())
            })
            .map_err(|failure| match failure {
                PhysicalDirtyReplacementError::Residency(reason) => reason,
                PhysicalDirtyReplacementError::Fill(never) => match never {},
            })
            .map(|dirty| (dirty, work))
    }
}

impl std::ops::Deref for LoadedPhysicalFrame {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.lease
    }
}
