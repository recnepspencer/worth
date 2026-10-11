//! One request pattern owns a complete maintenance pass and its typed preparation.
use super::work::MaintenanceWork;
use crate::execution::{self, RelationalExecutionDenialCause as Cause};
use crate::indexes::data::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, MapKernelFailure, MapStop, ScanOutcome,
    WorkCeilingDenial,
};
use worth_foundational::PartitionIdentity;

// Carries the pass's own additions, already reserved before allocation.
// Reading its charge is constant time, including for persistent index patches.
#[derive(Debug)]
pub(super) struct MaintenancePreparation<T> {
    pub(super) value: T,
    _memory: ExecutionMemoryReservation,
}

pub(super) fn run<T>(
    execution: &ExecutionRequest<'_, '_>,
    budget: DerivedIndexMaintenanceBudget,
    mut prepare: impl FnMut(&mut MaintenanceWork<'_, '_, '_>) -> Result<T, Denial>,
) -> Result<(MaintenancePreparation<T>, DerivedIndexMaintenanceWork), DerivedIndexMaintenanceDenial>
{
    let mut counts = DerivedIndexMaintenanceWork::default();
    execution
        .consult()
        .map_err(|denial| DerivedIndexMaintenanceDenial {
            kind: request_denial(denial),
            work: counts,
        })?;
    let result = execution
        .in_scope(|lease| {
            let mut memory = ExecutionMemoryReservation::reserve_in_scope(lease, 0)
                .map_err(|denial| Denial::Execution(execution::denial::lease_denial(denial)))?;
            let mut prepared = None;
            let scan = execution::admit_ordered_scan(vec![(PartitionIdentity::new(0), ())])
                .map_err(|stop| {
                    let crate::indexes::data::DerivedIndexExecutionDenialKind::Cause(cause) =
                        crate::indexes::data::DerivedIndexExecutionDenial::from(stop).kind;
                    Denial::Execution(cause)
                })?;
            // The scan owns only its unit receipt. The pass's additions are
            // reserved incrementally on the same request before allocation.
            let outcome = scan.run(lease, (), 0, 0, 0, 0, |_, _, context| {
                let mut work = MaintenanceWork::request_paid(budget, context, &mut memory);
                prepared = Some(prepare(&mut work));
                counts = work.counts();
                Ok::<_, MapKernelFailure<()>>(((), ()))
            });
            #[cfg(test)]
            LAST_ALLOCATION_BYTES.with(|bytes| bytes.set(memory.bytes()));
            match outcome {
                ScanOutcome::Complete { .. } => prepared
                    .expect("singleton scan prepared one maintenance pass")
                    .map(|value| MaintenancePreparation {
                        value,
                        _memory: memory,
                    }),
                ScanOutcome::Stopped { reason, .. } => Err(Denial::Execution(match reason {
                    MapStop::Admission(denial) => execution::denial::lease_denial(denial),
                    MapStop::WorkExhausted { .. } => Cause::WorkExhausted,
                    MapStop::Failure { cause, .. } => match cause {
                        MapKernelFailure::Stop(stop) => {
                            execution::kernel_failure(MapKernelFailure::Stop(stop))
                        }
                        MapKernelFailure::Panic => Cause::WorkerFailed,
                        MapKernelFailure::ResultCapacityExceeded => Cause::ResultCapacityExceeded,
                        MapKernelFailure::Domain(()) => {
                            unreachable!("maintenance returns native refusal as preparation")
                        }
                    },
                })),
            }
        })
        .map_err(request_denial)
        .and_then(|result| result);
    result
        .map(|value| (value, counts))
        .map_err(|kind| DerivedIndexMaintenanceDenial { kind, work: counts })
}

fn request_denial(denial: WorkCeilingDenial) -> Denial {
    Denial::Execution(match denial {
        WorkCeilingDenial::Admission(denial) => execution::denial::lease_denial(denial),
        WorkCeilingDenial::Stopped(stop) => execution::kernel_failure(MapKernelFailure::Stop(stop)),
        WorkCeilingDenial::Panicked => Cause::WorkerFailed,
    })
}

#[cfg(test)]
thread_local! {
    static LAST_ALLOCATION_BYTES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
pub(crate) fn last_allocation_bytes() -> u64 {
    LAST_ALLOCATION_BYTES.with(std::cell::Cell::get)
}
