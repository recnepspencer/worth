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
