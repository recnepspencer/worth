//! Advance original capture work while retaining the same physical reservation.
use super::*;

pub(in crate::domain_computation) enum RetainedStorageDenial {
    Overflow,
    Memory(worth_execution::MemoryLimitDenial),
}

/// Borrowed stage view; existing retained backing cannot be surrendered by it.
pub(in crate::domain_computation) struct RetainedCaptureMemory<'a> {
    reservation: &'a mut ExecutionMemoryReservation,
    base: u64,
}

impl RetainedCaptureMemory<'_> {
    pub(in crate::domain_computation) fn bytes(&self) -> u64 {
        self.reservation.bytes() - self.base
    }

    pub(in crate::domain_computation) fn resize(
        &mut self,
        additional: u64,
    ) -> Result<(), RetainedStorageDenial> {
        let total = self
            .base
            .checked_add(additional)
            .ok_or(RetainedStorageDenial::Overflow)?;
        self.reservation
            .resize(total)
            .map_err(RetainedStorageDenial::Memory)
    }
}

pub(in crate::domain_computation) fn run_retained_stage<Value, E>(
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
    kernel: impl FnOnce(
        &mut MapKernelContext<'_, '_>,
        Option<&mut RetainedCaptureMemory<'_>>,
    ) -> Result<Value, MapKernelFailure<E>>,
) -> Result<Value, CaptureStop<E>> {
    let mut settled = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        settled = Some(if context.has_accounting() {
            (|| {
                if memory.is_none() {
                    *memory = Some(
                        execution
                            .in_scope(|lease| {
                                ExecutionMemoryReservation::reserve_in_scope(lease, 0)
                            })
                            .map_err(CaptureStop::Request)?
                            .map_err(|cause| {
                                CaptureStop::Request(WorkCeilingDenial::Admission(cause))
                            })?,
                    );
                }
                let memory = memory
                    .as_mut()
                    .expect("accounted retention has its original ticket");
                let base = memory.bytes();
                let mut capture = RetainedCaptureMemory {
                    reservation: memory,
                    base,
                };
                kernel(context, Some(&mut capture))
                    .map_err(|cause| CaptureStop::Kernel(ScopeStop::Failure(cause)))
            })()
        } else {
            kernel(context, None).map_err(|cause| CaptureStop::Kernel(ScopeStop::Failure(cause)))
        });
    });
    let result = match (handoff, settled) {
        (Ok(_), Some(result)) => result,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(CaptureStop::Request(cause)),
        (Ok(_), None) => unreachable!("successful original retention invokes its capture"),
    };
    result
}

#[cfg(test)]
mod tests;
