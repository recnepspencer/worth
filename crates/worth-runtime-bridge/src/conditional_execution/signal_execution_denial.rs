//! Request checkpoint causes use the existing Bridge execution carrier.
use super::BridgeConditionalDenialKind as Kind;
use crate::error::BridgeExecutionDenial as Execution;
use worth_signal::facade::{
    SignalCheckpointDenial as Checkpoint, SignalError, SignalLeaseDenial as Lease,
};

pub(super) fn kind(error: &SignalError) -> Kind {
    let execution = match error {
        SignalError::ExecutionCheckpointStopped(cause) => match cause {
            Checkpoint::Cancelled => Execution::Cancelled,
            Checkpoint::DeadlineElapsed => Execution::DeadlineElapsed,
            Checkpoint::WorkCounterOverflow => Execution::WorkCounterOverflow,
            Checkpoint::WorkCeiling => Execution::WorkCeiling,
            Checkpoint::NestedStopped => Execution::NestedStopped,
        },
        SignalError::ExecutionAdmissionDenied(cause) => match cause {
            Lease::WorkerLimitExceedsParent => Execution::WorkerLimitExceedsParent,
            Lease::MemoryLimitExceedsParent => Execution::MemoryLimitExceedsParent,
            Lease::WorkLimitExceedsParent => Execution::WorkLimitExceedsParent,
            Lease::MemoryExhausted(cause) => Execution::MemoryExhausted(*cause),
            Lease::ChargedBytesOverflow => Execution::ChargedBytesOverflow,
            Lease::UnrelatedNestedLease => Execution::UnrelatedNestedLease,
            Lease::NoActiveExecutionScope => Execution::NoActiveExecutionScope,
            Lease::EquivalenceContractUnavailable => Execution::EquivalenceContractUnavailable,
        },
        SignalError::ExecutionPanicked => Execution::Panicked,
        SignalError::ExecutionStopped(stop) => {
            use worth_signal::facade::{
                SignalExecutionFailure as Failure, SignalExecutionStopReason as Stop,
            };
            return match stop.reason() {
                Stop::WorkExhausted { .. } => Kind::ExecutionDenied(Execution::WorkCeiling),
                Stop::Admission(cause) => kind(&SignalError::ExecutionAdmissionDenied(*cause)),
                Stop::PreparationMemoryExhausted { .. } => {
                    Kind::SignalExecution(super::BridgeSignalDenial::Error(error.clone()))
                }
                Stop::Failure { cause, .. } => match cause {
                    Failure::Domain(error) => kind(error),
                    Failure::Cancelled => Kind::ExecutionDenied(Execution::Cancelled),
                    Failure::DeadlineElapsed => Kind::ExecutionDenied(Execution::DeadlineElapsed),
                    Failure::WorkCounterOverflow => {
                        Kind::ExecutionDenied(Execution::WorkCounterOverflow)
                    }
                    Failure::WorkCeiling => Kind::ExecutionDenied(Execution::WorkCeiling),
                    Failure::NestedStopped => Kind::ExecutionDenied(Execution::NestedStopped),
                    Failure::Panic => Kind::ExecutionDenied(Execution::Panicked),
                    Failure::ResultCapacityExceeded => {
                        Kind::SignalExecution(super::BridgeSignalDenial::Error(error.clone()))
                    }
                },
            };
        }
        SignalError::RetainedStorageChargeOverflow
        | SignalError::RetainedStorageChargeUnderflow
        | SignalError::RetainedStorageHistoryUnavailable
        | SignalError::PreparationMemoryExhausted { .. }
        | SignalError::EvaluationStorageCapacityExhausted
        | SignalError::CheckedResultCapacityExceeded { .. }
        | SignalError::EvaluationStorageUnavailable
        | SignalError::SnapshotIndexUnavailable
        | SignalError::ConditionalEvaluationWorkExhausted { .. }
        | SignalError::UpstreamDependencyWorkExhausted { .. }
        | SignalError::WaiterResolutionWorkExhausted { .. }
        | SignalError::StaleHandle { .. }
        | SignalError::CycleDetected { .. }
        | SignalError::ScratchReentry { .. }
        | SignalError::ScratchMismatch { .. }
        | SignalError::ContractViolation { .. }
        | SignalError::TransactionFinished
        | SignalError::TransactionPoisoned
        | SignalError::EventFlushFailed { .. }
        | SignalError::IncompatibleSnapshot { .. }
        | SignalError::UnknownBranch { .. }
        | SignalError::BranchMergeFailed { .. }
        | SignalError::ManagedQueueBranchTransferDenied { .. }
        | SignalError::InvalidInput { .. }
        | SignalError::Internal { .. } => {
            return Kind::SignalExecution(super::BridgeSignalDenial::Error(error.clone()))
        }
    };
    Kind::ExecutionDenied(execution)
}
