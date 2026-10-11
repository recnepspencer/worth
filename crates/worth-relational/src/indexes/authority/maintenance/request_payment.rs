//! One request pattern owns a complete maintenance pass and its typed preparation.
use super::work::MaintenanceWork;
use crate::execution::{self, RelationalExecutionDenialCause as Cause};
use crate::indexes::data::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};
use worth_execution::{
    ChargedBytes, ExecutionRequest, MapKernelFailure, MapStop, ScanOutcome, WorkCeilingDenial,
};
use worth_foundational::PartitionIdentity;

// A native refusal (e.g. cold-required) finishes the pass and can be retried.
// Execution stops still terminate the pattern at their exact charge boundary.
struct MaintenancePreparation<T>(Result<T, Denial>);
impl<T: ChargedBytes> ChargedBytes for MaintenancePreparation<T> {
    fn additional_charged_bytes(&self) -> u64 {
        match &self.0 {
            Ok(value) => value.additional_charged_bytes(),
            Err(denial) => denial.owned_allocation_capacity_bytes(),
        }
    }
}

pub(super) fn run<T: ChargedBytes>(
    execution: &ExecutionRequest<'_, '_>,
    budget: DerivedIndexMaintenanceBudget,
    mut prepare: impl FnMut(&mut MaintenanceWork<'_, '_, '_>) -> Result<T, Denial>,
) -> Result<(T, DerivedIndexMaintenanceWork), DerivedIndexMaintenanceDenial> {
    let mut counts = DerivedIndexMaintenanceWork::default();
    execution
        .consult()
        .map_err(|denial| DerivedIndexMaintenanceDenial {
            kind: request_denial(denial),
            work: counts,
        })?;
    let result = execution
        .in_scope(|lease| {
            let scan = execution::admit_ordered_scan(vec![(PartitionIdentity::new(0), ())])
                .map_err(|stop| {
                    let crate::indexes::data::DerivedIndexExecutionDenialKind::Cause(cause) =
                        crate::indexes::data::DerivedIndexExecutionDenial::from(stop).kind;
                    Denial::Execution(cause)
                })?;
            let output_capacity = execution.memory_limit() / 4;
            let outcome = scan.run(lease, (), 0, output_capacity, 0, 0, |_, _, context| {
                let mut work = MaintenanceWork::request_paid(budget, context);
                let result = prepare(&mut work);
                counts = work.counts;
                Ok::<_, MapKernelFailure<()>>(((), MaintenancePreparation(result)))
            });
            match outcome {
                ScanOutcome::Complete { mut prefixes, .. } => {
                    prefixes
                        .pop()
                        .expect("one maintenance preparation for the singleton scan")
                        .0
                }
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
