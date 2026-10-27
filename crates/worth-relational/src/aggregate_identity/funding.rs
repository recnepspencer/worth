//! Original Native accounting posture and retained consequence preparation custody.
use super::observations::{Acquisition, AggregateReadObservations};
use crate::transactions::data::TransactionCommitError;
use worth_execution::{ChargedBytes, ExecutionMemoryReservation, MapKernelFailure, ScopeStop};
mod failure;
pub(super) use failure::{FundingFailure, PlanDenial};
#[cfg(test)]
mod tests;
pub(super) type BuildResult<T> = Result<T, MapKernelFailure<PlanDenial>>;

pub(super) struct AggregateWork<'a, 'scope, 'authority, 'memory> {
    context: &'a mut worth_execution::MapKernelContext<'scope, 'authority>,
    memory: Option<&'a mut worth_execution::ExecutionResultMemory<'memory>>,
    claimed: u64,
    read_checkpoint: &'a mut dyn FnMut(u64) -> BuildResult<()>,
    observations: Option<&'a mut AggregateReadObservations>,
}
impl AggregateWork<'_, '_, '_, '_> {
    pub(super) fn checkpoint(&mut self, units: u64) -> BuildResult<()> {
        if self.context.has_accounting() {
            self.context.checkpoint(units)
        } else {
            self.context.poll()
        }
        .map_err(MapKernelFailure::Stop)?;
        let result = (self.read_checkpoint)(units);
        self.settle(result)
    }
    pub(super) fn poll(&mut self) -> BuildResult<()> {
        self.context.poll().map_err(MapKernelFailure::Stop)
    }
    fn settle<T>(&mut self, result: BuildResult<T>) -> BuildResult<T> {
        match (self.context.poll(), result) {
            // The callback began at a clean original checkpoint. Its actual
            // execution refusal may be promoted by its own nested Native work.
            (
                Err(worth_execution::MapKernelStop::NestedStopped),
                Err(MapKernelFailure::Domain(PlanDenial::Execution(cause))),
            ) => Err(MapKernelFailure::Domain(PlanDenial::Execution(cause))),
            (Err(stop), _) => Err(MapKernelFailure::Stop(stop)),
            (Ok(()), result) => result,
        }
    }
    pub(super) fn acquired(&mut self, acquisition: Acquisition) -> BuildResult<()> {
        if let Some(observations) = self.observations.as_mut() {
            observations.acquired(acquisition)?;
        }
        Ok(())
    }
    pub(super) fn retain(&mut self, bytes: u64) -> BuildResult<()> {
        self.checkpoint(1)?;
        self.claimed = self
            .claimed
            .checked_add(bytes)
            .ok_or(MapKernelFailure::Domain(PlanDenial::Execution(
                crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
            )))?;
        if let Some(memory) = self.memory.as_mut() {
            memory.resize(self.claimed).map_err(|denial| {
                MapKernelFailure::Domain(PlanDenial::Execution(
                    crate::execution::denial::lease_denial(
                        worth_execution::LeaseDenial::MemoryExhausted(denial),
                    ),
                ))
            })?;
        }
        Ok(())
    }
    pub(super) fn invalid<T>(&self, denial: PlanDenial) -> BuildResult<T> {
        Err(MapKernelFailure::Domain(denial))
    }
}

pub(super) fn run<T: ChargedBytes>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    budget: Option<&crate::execution::RequestWorkBudget>,
    build: impl FnOnce(&mut AggregateWork<'_, '_, '_, '_>) -> BuildResult<T>,
) -> Result<(T, Option<ExecutionMemoryReservation>), TransactionCommitError> {
    run_core(execution, budget, &mut |_| Ok(()), None, build).map_err(FundingFailure::into_commit)
}

pub(super) fn run_custom_read<T: ChargedBytes>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    allowance: &mut dyn FnMut(u64) -> Result<(), crate::execution::RelationalExecutionDenialCause>,
    build: impl FnOnce(&mut AggregateWork<'_, '_, '_, '_>) -> BuildResult<T>,
) -> Result<(T, Option<ExecutionMemoryReservation>), TransactionCommitError> {
    let mut checkpoint = |units| {
        allowance(units).map_err(|cause| MapKernelFailure::Domain(PlanDenial::Execution(cause)))
    };
    run_core(execution, None, &mut checkpoint, None, build).map_err(FundingFailure::into_commit)
}

enum ReadOutcome<T> {
    Value(T),
    Declined(PlanDenial),
}
impl<T: ChargedBytes> ChargedBytes for ReadOutcome<T> {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Value(value) => value.additional_charged_bytes(),
            Self::Declined(denial) => denial.additional_charged_bytes(),
        }
    }
}

/// Read-owned domain declines are data, not failed framework computations.
/// Request/resource failures keep their original stopping posture.
pub(super) fn run_read<T: ChargedBytes>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    checkpoint: &mut dyn FnMut(u64) -> BuildResult<()>,
    observations: &mut AggregateReadObservations,
    build: impl FnOnce(&mut AggregateWork<'_, '_, '_, '_>) -> BuildResult<T>,
) -> Result<(T, Option<ExecutionMemoryReservation>), FundingFailure> {
    let (outcome, retention) = run_core(execution, None, checkpoint, Some(observations), |work| {
        work.poll()?;
        let result = build(work);
        match work.settle(result) {
            Ok(value) => Ok(ReadOutcome::Value(value)),
            Err(MapKernelFailure::Domain(denial))
                if !matches!(denial, PlanDenial::Execution(_)) =>
            {
                Ok(ReadOutcome::Declined(denial))
            }
            Err(failure) => Err(failure),
        }
    })?;
    match outcome {
        ReadOutcome::Value(value) => Ok((value, retention)),
        ReadOutcome::Declined(denial) => Err(FundingFailure::Domain(denial)),
    }
}

fn run_core<T: ChargedBytes>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    budget: Option<&crate::execution::RequestWorkBudget>,
    checkpoint: &mut dyn FnMut(u64) -> BuildResult<()>,
    mut observations: Option<&mut AggregateReadObservations>,
    build: impl FnOnce(&mut AggregateWork<'_, '_, '_, '_>) -> BuildResult<T>,
) -> Result<(T, Option<ExecutionMemoryReservation>), FundingFailure> {
    let mut build = Some(build);
    enum NativeStage<T> {
        Accounted,
        Unaccounted(T),
    }
    // Probe the actual Native ancestor without enclosing the measured stage:
    // an outer Native handoff must not replace a typed measured stop with
    // NestedStopped after the original report has already been settled.
    let (stage, _) = execution
        .run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                return Ok(NativeStage::Accounted);
            }
            let mut work = AggregateWork {
                context,
                memory: None,
                claimed: 0,
                read_checkpoint: &mut *checkpoint,
                observations: observations.as_deref_mut(),
            };
            build.take().expect("one aggregate preparation")(&mut work)
                .map(NativeStage::Unaccounted)
                .map_err(failure::kernel)
        })
        .map_err(|denial| FundingFailure::Execution(denial.into()))?;
    if let NativeStage::Unaccounted(value) = stage? {
        return Ok((value, None));
    }
    let mut settled = None;
    let handoff = execution.in_scope(|lease| {
        settled = Some((|| {
            let mut retention =
                ExecutionMemoryReservation::reserve_in_scope(lease, 0).map_err(failure::lease)?;
            let mut invoke = |ceiling| {
                execution.run_measured_result(
                    worth_execution::ExecutionWorkCeiling::new(ceiling),
                    0,
                    &mut retention,
                    |context, memory| {
                        let mut work = AggregateWork {
                            context,
                            memory: Some(memory),
                            claimed: 0,
                            read_checkpoint: &mut *checkpoint,
                            observations: observations.as_deref_mut(),
                        };
                        build.take().expect("one aggregate preparation")(&mut work)
                    },
                )
            };
            let outcome = match lease {
                Some(lease) => crate::execution::run_with_remaining_request_work(
                    lease,
                    budget,
                    |child| invoke(child.policy().work_ceiling()),
                    |outcome| outcome.report,
                ),
                None => invoke(execution.work_ceiling()),
            };
            match outcome.result {
                Ok(value) => Ok((value, Some(retention))),
                Err(ScopeStop::Admission(denial)) => Err(failure::lease(denial)),
                Err(ScopeStop::Failure(cause)) => Err(failure::kernel(cause)),
            }
        })());
    });
    match (handoff, settled) {
        (Ok(()), Some(result)) => result,
        // A measured refusal has already settled its report. Native serial
        // handoff promotes that refusal to NestedStopped; keep its typed cause.
        (
            Err(worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::NestedStopped,
            )),
            Some(Err(error)),
        ) => Err(error),
        (Err(denial), _) => Err(FundingFailure::Execution(denial.into())),
        (Ok(()), None) => unreachable!("successful scope invokes aggregate stage"),
    }
}
