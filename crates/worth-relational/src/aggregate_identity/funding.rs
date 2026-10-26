//! Original Native accounting posture and retained consequence preparation custody.
use super::AggregateTransitionDenial;
use crate::transactions::data::TransactionCommitError;
use worth_execution::{ChargedBytes, ExecutionMemoryReservation, MapKernelFailure, ScopeStop};
#[cfg(test)]
mod tests;
#[derive(Debug)]
pub(super) enum PlanDenial {
    Execution(crate::execution::RelationalExecutionDenialCause),
    OrdinaryMetadataWrite,
    InvalidPermutation,
    InvalidStoredIdentity,
    IncompleteStoredMembership,
    Transition(AggregateTransitionDenial),
}
impl From<AggregateTransitionDenial> for PlanDenial {
    fn from(value: AggregateTransitionDenial) -> Self {
        Self::Transition(value)
    }
}
impl ChargedBytes for PlanDenial {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
pub(super) type BuildResult<T> = Result<T, MapKernelFailure<PlanDenial>>;
impl From<AggregateTransitionDenial> for MapKernelFailure<PlanDenial> {
    fn from(value: AggregateTransitionDenial) -> Self {
        Self::Domain(value.into())
    }
}

pub(super) struct AggregateWork<'a, 'scope, 'authority, 'memory> {
    context: &'a mut worth_execution::MapKernelContext<'scope, 'authority>,
    memory: Option<&'a mut worth_execution::ExecutionResultMemory<'memory>>,
    claimed: u64,
}
impl AggregateWork<'_, '_, '_, '_> {
    pub(super) fn checkpoint(&mut self, units: u64) -> BuildResult<()> {
        if self.context.has_accounting() {
            self.context.checkpoint(units)
        } else {
            self.context.poll()
        }
        .map_err(MapKernelFailure::Stop)
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
            };
            build.take().expect("one aggregate preparation")(&mut work)
                .map(NativeStage::Unaccounted)
                .map_err(failure)
        })
        .map_err(|denial| execution_error(denial.into()))?;
    if let NativeStage::Unaccounted(value) = stage? {
        return Ok((value, None));
    }
    let mut settled = None;
    let handoff = execution.in_scope(|lease| {
        settled = Some((|| {
            let mut retention =
                ExecutionMemoryReservation::reserve_in_scope(lease, 0).map_err(lease_error)?;
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
                Err(ScopeStop::Admission(denial)) => Err(lease_error(denial)),
                Err(ScopeStop::Failure(cause)) => Err(failure(cause)),
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
        (Err(denial), _) => Err(execution_error(denial.into())),
        (Ok(()), None) => unreachable!("successful scope invokes aggregate stage"),
    }
}

fn failure(cause: MapKernelFailure<PlanDenial>) -> TransactionCommitError {
    use crate::execution::RelationalExecutionDenialCause as Cause;
    match cause {
        MapKernelFailure::Domain(denial) => invalid_plan(denial),
        MapKernelFailure::Stop(stop) => execution_error(stop.into()),
        MapKernelFailure::ResultCapacityExceeded => execution_error(Cause::ResultCapacityExceeded),
        MapKernelFailure::Panic => execution_error(Cause::WorkerFailed),
    }
}
fn invalid_plan(denial: PlanDenial) -> TransactionCommitError {
    if let PlanDenial::Execution(cause) = denial {
        return execution_error(cause);
    }
    let detail = match denial {
        PlanDenial::Transition(denial) => {
            format!("Native aggregate identity transition denied: {denial:?}")
        }
        other => format!("Native aggregate identity preparation denied: {other:?}"),
    };
    TransactionCommitError::conflict(crate::transactions::data::CommitConflict::new(
        crate::transactions::data::ConflictClass::KindSchemaMismatch { detail },
    ))
}
fn lease_error(denial: worth_execution::LeaseDenial) -> TransactionCommitError {
    execution_error(crate::execution::denial::lease_denial(denial))
}
fn execution_error(
    cause: crate::execution::RelationalExecutionDenialCause,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause),
        partition_identity: None,
    })
}
