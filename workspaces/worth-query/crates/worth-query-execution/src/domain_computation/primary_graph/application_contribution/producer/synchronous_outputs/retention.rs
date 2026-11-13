//! Fund captured containers through the original request's retained-stage owner.
use super::*;
use crate::domain_computation::primary_graph::{
    admitted_capture::{self, CaptureStop, RetainedStorageDenial},
    WorthQueryApplicationAttemptDenial as Denial,
};
use worth_execution::{LeaseDenial, MapKernelFailure, ScopeStop, WorkCeilingDenial};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(in crate::domain_computation::primary_graph) struct CaptureRefusal {
    subject: &'static str,
    cause: Cause,
}
impl CaptureRefusal {
    pub(super) fn denial(&self) -> Denial {
        Denial::relational_read_stopped(self.cause, self.subject)
    }
}
impl CapturedSynchronousOutputs {
    pub(super) fn capture(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        subject: &'static str,
        bytes: usize,
        build: impl FnOnce() -> Box<dyn std::any::Any + Send>,
    ) -> Self {
        let refused = |cause| Self::Refused(CaptureRefusal { subject, cause });
        let Ok(bytes) = u64::try_from(bytes) else {
            return refused(Cause::ChargedBytesOverflow);
        };
        let mut memory = None;
        let root =
            admitted_capture::run_retained_stage(execution, &mut memory, |context, memory| {
                admitted_capture::checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
                if let Some(memory) = memory {
                    memory.resize(bytes).map_err(|cause| {
                        MapKernelFailure::Domain(match cause {
                            RetainedStorageDenial::Overflow => Cause::ChargedBytesOverflow,
                            RetainedStorageDenial::Memory(cause) => {
                                WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(cause))
                                    .into()
                            }
                        })
                    })?;
                }
                Ok(build())
            });
        match root {
            Ok(root) => Self::Prepared {
                root,
                source_recipe: None,
                _memory: memory,
            },
            Err(CaptureStop::Request(cause)) => refused(cause.into()),
            Err(CaptureStop::Kernel(ScopeStop::Admission(cause))) => {
                refused(WorkCeilingDenial::Admission(cause).into())
            }
            Err(CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Stop(cause)))) => {
                refused(cause.into())
            }
            Err(CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(cause)))) => {
                refused(cause)
            }
            Err(CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Panic))) => {
                refused(Cause::WorkerFailed)
            }
            Err(CaptureStop::Kernel(ScopeStop::Failure(
                MapKernelFailure::ResultCapacityExceeded,
            ))) => refused(Cause::ResultCapacityExceeded),
        }
    }
}
