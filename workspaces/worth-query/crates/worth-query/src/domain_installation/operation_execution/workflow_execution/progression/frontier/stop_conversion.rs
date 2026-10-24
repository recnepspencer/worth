//! Every authority stop keeps its cause at the workflow boundary.
use super::{
    WorkflowFrontierFailure, WorthQueryWorkflowAdvanceDenialKind as Denial,
    WorthQueryWorkflowStageComputed,
};
use worth_execution::{LeaseDenial, MapKernelFailure, MapKernelStop, MapStop, WorkCeilingDenial};

fn admission(cause: LeaseDenial) -> WorkflowFrontierFailure {
    let cause = match cause {
        LeaseDenial::WorkerLimitExceedsParent
        | LeaseDenial::MemoryLimitExceedsParent
        | LeaseDenial::WorkLimitExceedsParent
        | LeaseDenial::MemoryExhausted(_)
        | LeaseDenial::ChargedBytesOverflow
        | LeaseDenial::UnrelatedNestedLease
        | LeaseDenial::NoActiveExecutionScope
        | LeaseDenial::EquivalenceContractUnavailable => cause,
    };
    WorkflowFrontierFailure::Denied(Denial::ComputationAdmission(cause))
}
fn checkpoint(cause: MapKernelStop, stage_identity: Option<String>) -> WorkflowFrontierFailure {
    match cause {
        MapKernelStop::Cancelled => {
            WorkflowFrontierFailure::Denied(Denial::ComputationCancelled { stage_identity })
        }
        MapKernelStop::DeadlineElapsed => {
            WorkflowFrontierFailure::Denied(Denial::ComputationDeadline { stage_identity })
        }
        MapKernelStop::WorkCounterOverflow | MapKernelStop::WorkCeiling => {
            WorkflowFrontierFailure::Denied(Denial::ComputationWorkExhausted {
                stage_identity,
                cause,
            })
        }
        MapKernelStop::NestedStopped => {
            WorkflowFrontierFailure::Denied(Denial::ComputationNestedStopped { stage_identity })
        }
        MapKernelStop::AccountingUnavailable => {
            WorkflowFrontierFailure::Denied(Denial::ComputationAccountingUnavailable {
                stage_identity,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_only_accounting_stop_keeps_the_workflow_resource_cause() {
        use worth_execution::{
            CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest,
        };
        use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
        let lease = ExecutionResourceLease::serial(LeaseRequest {
            policy: ExecutionRequestPolicy::for_workers(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                std::num::NonZeroUsize::MIN,
            ),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
        let stop = ExecutionRequest::leased(&lease)
            .run_native_with_checkpoint(|context| context.checkpoint(1))
            .unwrap_err();
        assert!(matches!(
            scope(stop),
            WorkflowFrontierFailure::Denied(Denial::ComputationAccountingUnavailable {
                stage_identity: None
            })
        ));
    }

    #[test]
    fn unavailable_accounting_retains_its_own_workflow_stop() {
        let stopped = scope(WorkCeilingDenial::Stopped(
            MapKernelStop::AccountingUnavailable,
        ));
        assert!(matches!(
            stopped,
            WorkflowFrontierFailure::Denied(Denial::ComputationAccountingUnavailable {
                stage_identity: None
            })
        ));
    }
}
pub(super) fn scope(cause: WorkCeilingDenial) -> WorkflowFrontierFailure {
    match cause {
        WorkCeilingDenial::Admission(cause) => admission(cause),
        WorkCeilingDenial::Stopped(cause) => checkpoint(cause, None),
        WorkCeilingDenial::Panicked => WorkflowFrontierFailure::Denied(Denial::ComputationPanic {
            stage_identity: None,
        }),
    }
}
pub(super) fn map(
    cause: MapStop<WorthQueryWorkflowStageComputed>,
    stage_identity: Option<String>,
) -> WorkflowFrontierFailure {
    match cause {
        MapStop::Admission(cause) => admission(cause),
        MapStop::Allocation(cause) => {
            WorkflowFrontierFailure::Denied(Denial::ComputationAllocation {
                stage_identity,
                cause,
            })
        }
        MapStop::WorkExhausted { .. } => checkpoint(MapKernelStop::WorkCeiling, stage_identity),
        MapStop::Failure { cause, .. } => match cause {
            MapKernelFailure::Stop(cause) => checkpoint(cause, stage_identity),
            MapKernelFailure::Domain(result) => WorkflowFrontierFailure::Domain(result),
            MapKernelFailure::Panic => {
                WorkflowFrontierFailure::Denied(Denial::ComputationPanic { stage_identity })
            }
            MapKernelFailure::ResultCapacityExceeded => {
                WorkflowFrontierFailure::Denied(Denial::ComputationResultCapacity {
                    stage_identity,
                })
            }
        },
    }
}
