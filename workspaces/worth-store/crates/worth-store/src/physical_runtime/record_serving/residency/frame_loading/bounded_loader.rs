use worth_store_buffer_pool::{
    PhysicalBoundedFrameAccess, PhysicalBoundedFrameKey, PhysicalFrameAccess,
    PhysicalFrameFaultError, PhysicalFrameKey, PhysicalResidencyPool,
};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

#[cfg(feature = "certification-test-authority")]
use super::read_source::FrameReadSourceFailure;
use super::read_source::{
    frame_source_failure, frame_source_fault, FrameReadSource, PreparedFrameRead,
};
use super::{
    ExactFrameSourceExtent, FrameLoadFailure, FrameLoadFailureKind, FrameLoadFaultCause,
    FrameLoadPort, LoadedPhysicalFrame, PhysicalFrameAccessOrigin,
};
use crate::physical_runtime::record_serving::residency::frame_work_trace::FrameWorkTrace;

#[derive(Clone)]
pub(in crate::physical_runtime::record_serving) struct BoundedFrameLoader {
    pub(super) pool: PhysicalResidencyPool,
}

struct PreparedExactFrameRead<'source> {
    preceding_work: FrameWorkTrace,
    prepared: Box<dyn PreparedFrameRead + 'source>,
}

impl BoundedFrameLoader {
    pub(in crate::physical_runtime::record_serving) const fn new(
        pool: PhysicalResidencyPool,
    ) -> Self {
        Self { pool }
    }
}

impl FrameLoadPort for BoundedFrameLoader {
    fn load_exact(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        source: &dyn FrameReadSource,
        artifact: RecordArtifactFile,
        offset: u64,
        length: u32,
        source_extent: ExactFrameSourceExtent,
    ) -> Result<LoadedPhysicalFrame, FrameLoadFailure> {
        let coordinate = RecordFrameCoordinate::new(artifact, offset, length).ok_or(
            FrameLoadFailure::new(FrameLoadFailureKind::InvalidCoordinate),
        )?;
        if let ExactFrameSourceExtent::ArenaRange(range) = source_extent {
            let inside = artifact
                == RecordArtifactFile::ExtentArena {
                    arena: range.arena().get(),
                }
                && offset >= range.offset()
                && offset
                    .checked_add(u64::from(length))
                    .is_some_and(|end| end <= range.end());
            if !inside {
                return Err(FrameLoadFailure::new(
                    FrameLoadFailureKind::InvalidCoordinate,
                ));
            }
        }
        let key = PhysicalFrameKey::new(self.pool.store_identity(), coordinate);
        let access = self
            .pool
            .access_frame(allocation, key)
            .map_err(|reason| FrameLoadFailure::new(FrameLoadFailureKind::Residency(reason)))?;
        self.load_admitted_exact_with_source_extent(coordinate, access, || {
            let preceding_work = validate_source_extent(source, artifact, source_extent)?;
            let prepared = source
                .prepare_exact(artifact, offset, length)
                .map_err(frame_source_failure)?;
            Ok(PreparedExactFrameRead {
                preceding_work,
                prepared,
            })
        })
    }

    fn load_bounded(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        source: &dyn FrameReadSource,
        artifact: RecordArtifactFile,
        limit: u32,
    ) -> Result<LoadedPhysicalFrame, FrameLoadFailure> {
        let limit = std::num::NonZeroU32::new(limit).ok_or(FrameLoadFailure::new(
            FrameLoadFailureKind::AccessLimitExceeded,
        ))?;
        let key = PhysicalBoundedFrameKey::new(self.pool.store_identity(), artifact, limit);
        let access = self
            .pool
            .access_bounded_frame(allocation, key)
            .map_err(|reason| FrameLoadFailure::new(FrameLoadFailureKind::Residency(reason)))?;
        let work = std::cell::Cell::new(FrameWorkTrace::none());
        let mut projection_failure = None;
        let (lease, origin) = match access {
            PhysicalBoundedFrameAccess::Hit(lease) => (lease, PhysicalFrameAccessOrigin::Hit),
            PhysicalBoundedFrameAccess::Coalesced(waiter) => (
                waiter.wait().map_err(|terminal| {
                    FrameLoadFailure::new(FrameLoadFailureKind::CoalescedFault(terminal))
                })?,
                PhysicalFrameAccessOrigin::Coalesced,
            ),
            PhysicalBoundedFrameAccess::Fault(owner) => (
                owner
                    .load_observed(
                        |admitted_limit| {
                            let length =
                                source.file_length(artifact).map_err(frame_source_failure)?;
                            if length.bytes() == 0
                                || length.bytes() > u64::from(admitted_limit)
                                || length.bytes() > u64::from(u32::MAX)
                            {
                                let observed = length.reject_structural_damage();
                                work.set(observed);
                                return Err(FrameLoadFailure::new(
                                    FrameLoadFailureKind::AccessLimitExceeded,
                                )
                                .with_complete_work_trace(observed));
                            }
                            work.set(length.work_trace());
                            Ok(length.bytes() as u32)
                        },
                        |target: &mut [u8]| {
                            let prepared = source
                                .prepare_exact(artifact, 0, target.len() as u32)
                                .map_err(|failure| {
                                frame_source_failure(failure).preceded_by(work.get())
                            })?;
                            let identity = prepared.identity();
                            work.set(work.get().then(FrameWorkTrace::one(identity)));
                            projection_failure = prepared.execute(target).map_err(|failure| {
                                frame_source_failure(failure).with_complete_work_trace(work.get())
                            })?;
                            Ok(identity.map(allocation_operation))
                        },
                    )
                    .map_err(|failure| bounded_fault_failure(failure, work.get()))?,
                PhysicalFrameAccessOrigin::Fault,
            ),
        };
        let coordinate = lease.key().coordinate();
        LoadedPhysicalFrame::bind(
            self.pool.store_identity(),
            coordinate,
            lease,
            origin,
            work.get(),
            projection_failure,
        )
    }
}

impl BoundedFrameLoader {
    #[cfg(feature = "certification-test-authority")]
    pub(super) fn load_admitted_exact<'source>(
        &self,
        coordinate: RecordFrameCoordinate,
        access: PhysicalFrameAccess,
        prepare: impl FnOnce() -> Result<Box<dyn PreparedFrameRead + 'source>, FrameReadSourceFailure>,
    ) -> Result<LoadedPhysicalFrame, FrameLoadFailure> {
        self.load_admitted_exact_with_source_extent(coordinate, access, || {
            let prepared = prepare().map_err(frame_source_failure)?;
            Ok(PreparedExactFrameRead {
                preceding_work: FrameWorkTrace::none(),
                prepared,
            })
        })
    }

    fn load_admitted_exact_with_source_extent<'source>(
        &self,
        coordinate: RecordFrameCoordinate,
        access: PhysicalFrameAccess,
        prepare: impl FnOnce() -> Result<PreparedExactFrameRead<'source>, FrameLoadFailure>,
    ) -> Result<LoadedPhysicalFrame, FrameLoadFailure> {
        let (lease, origin, work, projection_failure) = match access {
            PhysicalFrameAccess::Hit(lease) => (
                lease,
                PhysicalFrameAccessOrigin::Hit,
                FrameWorkTrace::none(),
                None,
            ),
            PhysicalFrameAccess::Coalesced(waiter) => {
                let terminal = waiter.wait().map_err(|terminal| {
                    FrameLoadFailure::new(FrameLoadFailureKind::CoalescedFault(terminal))
                })?;
                (
                    terminal,
                    PhysicalFrameAccessOrigin::Coalesced,
                    FrameWorkTrace::none(),
                    None,
                )
            }
            PhysicalFrameAccess::Fault(fault) => {
                let prepared = match prepare() {
                    Ok(prepared) => prepared,
                    Err(failure) => {
                        let terminal = fault.reject_before_source();
                        return Err(exact_preparation_failure(failure, terminal));
                    }
                };
                let work = prepared
                    .preceding_work
                    .then(FrameWorkTrace::one(prepared.prepared.identity()));
                let operation = prepared.prepared.identity().map(allocation_operation);
                let mut projection_failure = None;
                let lease = fault
                    .load_observed(operation, |target| {
                        projection_failure = prepared.prepared.execute(target)?;
                        Ok(())
                    })
                    .map_err(|failure| match failure {
                        PhysicalFrameFaultError::Residency { terminal, denial } => {
                            FrameLoadFailure::new(FrameLoadFailureKind::FaultTerminated {
                                terminal,
                                cause: FrameLoadFaultCause::Residency(denial),
                            })
                            .with_complete_work_trace(work)
                        }
                        PhysicalFrameFaultError::Source { terminal, failure } => {
                            frame_source_fault(failure, terminal).with_complete_work_trace(work)
                        }
                    })?;
                (
                    lease,
                    PhysicalFrameAccessOrigin::Fault,
                    work,
                    projection_failure,
                )
            }
        };
        LoadedPhysicalFrame::bind(
            self.pool.store_identity(),
            coordinate,
            lease,
            origin,
            work,
            projection_failure,
        )
    }
}

fn allocation_operation(
    identity: crate::physical_runtime::PhysicalWorkIdentity,
) -> worth_store_buffer_pool::PhysicalResidencyAllocationOperation {
    worth_store_buffer_pool::PhysicalResidencyAllocationOperation::new(
        std::num::NonZeroU64::new(identity.operation().get())
            .expect("physical operation identity is nonzero"),
    )
}

fn validate_source_extent(
    source: &dyn FrameReadSource,
    artifact: RecordArtifactFile,
    source_extent: ExactFrameSourceExtent,
) -> Result<FrameWorkTrace, FrameLoadFailure> {
    match source_extent {
        ExactFrameSourceExtent::ArenaRange(_) => Ok(FrameWorkTrace::none()),
        #[cfg(feature = "certification-test-authority")]
        ExactFrameSourceExtent::CoordinateOnly => Ok(FrameWorkTrace::none()),
        ExactFrameSourceExtent::CompleteArtifact(expected) => {
            let observed = source.file_length(artifact).map_err(frame_source_failure)?;
            let work = observed.work_trace();
            if observed.bytes() != expected.get() {
                observed.reject_structural_damage();
                return Err(
                    FrameLoadFailure::new(FrameLoadFailureKind::ArtifactLengthMismatch)
                        .with_complete_work_trace(work),
                );
            }
            Ok(work)
        }
    }
}

fn exact_preparation_failure(
    failure: FrameLoadFailure,
    terminal: worth_store_buffer_pool::PhysicalFrameLoadTerminal,
) -> FrameLoadFailure {
    let cause = match failure.kind() {
        FrameLoadFailureKind::Backend(cause) => Some(FrameLoadFaultCause::Backend(cause)),
        FrameLoadFailureKind::Work(cause) => Some(FrameLoadFaultCause::Work(cause)),
        FrameLoadFailureKind::Residency(cause) => Some(FrameLoadFaultCause::Residency(cause)),
        _ => None,
    };
    cause.map_or(failure, |cause| {
        FrameLoadFailure::new(FrameLoadFailureKind::FaultTerminated { terminal, cause })
            .with_complete_work_trace(failure.work_trace())
    })
}

fn bounded_fault_failure(
    failure: PhysicalFrameFaultError<FrameLoadFailure>,
    work: FrameWorkTrace,
) -> FrameLoadFailure {
    match failure {
        PhysicalFrameFaultError::Residency { terminal, denial } => {
            FrameLoadFailure::new(FrameLoadFailureKind::FaultTerminated {
                terminal,
                cause: FrameLoadFaultCause::Residency(denial),
            })
            .with_complete_work_trace(work)
        }
        PhysicalFrameFaultError::Source { terminal, failure } => match failure.kind() {
            FrameLoadFailureKind::Backend(cause) => {
                FrameLoadFailure::new(FrameLoadFailureKind::FaultTerminated {
                    terminal,
                    cause: FrameLoadFaultCause::Backend(cause),
                })
                .with_complete_work_trace(failure.work_trace())
            }
            FrameLoadFailureKind::Work(cause) => {
                FrameLoadFailure::new(FrameLoadFailureKind::FaultTerminated {
                    terminal,
                    cause: FrameLoadFaultCause::Work(cause),
                })
                .with_complete_work_trace(failure.work_trace())
            }
            FrameLoadFailureKind::Residency(cause) => {
                FrameLoadFailure::new(FrameLoadFailureKind::FaultTerminated {
                    terminal,
                    cause: FrameLoadFaultCause::Residency(cause),
                })
                .with_complete_work_trace(failure.work_trace())
            }
            _ => failure,
        },
    }
}
