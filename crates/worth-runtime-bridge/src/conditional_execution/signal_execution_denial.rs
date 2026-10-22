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
            Checkpoint::AccountingUnavailable => Execution::AccountingUnavailable,
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
                Stop::PreparationMemoryExhausted { .. } | Stop::Allocation(_) => {
                    Kind::SignalExecution(super::BridgeSignalDenial::Error(error.clone()))
                }
                Stop::Failure { cause, .. } => match cause {
                    Failure::Domain(error) => kind(error),
                    Failure::Cancelled => Kind::ExecutionDenied(Execution::Cancelled),
                    Failure::DeadlineElapsed => Kind::ExecutionDenied(Execution::DeadlineElapsed),
                    Failure::WorkCounterOverflow => {
                        Kind::ExecutionDenied(Execution::WorkCounterOverflow)
                    }
                    Failure::AccountingUnavailable => {
                        Kind::ExecutionDenied(Execution::AccountingUnavailable)
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
        | SignalError::MissingBoundedInputsAtEpochAdmission { .. }
        | SignalError::MissingBoundedInputsAtEvaluation { .. }
        | SignalError::InvalidInput { .. }
        | SignalError::Internal { .. } => {
            return Kind::SignalExecution(super::BridgeSignalDenial::Error(error.clone()))
        }
    };
    Kind::ExecutionDenied(execution)
}

#[cfg(test)]
mod footprint_tests {
    use super::*;
    #[test]
    fn unsupported_billing_retains_its_distinct_signal_cause() {
        assert_eq!(
            kind(&SignalError::ExecutionCheckpointStopped(
                Checkpoint::AccountingUnavailable
            )),
            Kind::ExecutionDenied(Execution::AccountingUnavailable)
        );
        assert_eq!(
            Execution::from(worth_execution::MapKernelStop::AccountingUnavailable),
            Execution::AccountingUnavailable
        );
        assert_eq!(
            Execution::AccountingUnavailable.label(),
            "accounting-unavailable"
        );
    }
    #[test]
    fn a_missing_footprint_keeps_its_typed_signal_site() {
        let mut graph = worth_signal::facade::SignalGraph::new();
        let node = graph.node().build();
        for error in [
            SignalError::MissingBoundedInputsAtEpochAdmission { node },
            SignalError::MissingBoundedInputsAtEvaluation { node },
        ] {
            assert_eq!(
                kind(&error),
                Kind::SignalExecution(super::super::BridgeSignalDenial::Error(error.clone()))
            );
        }
    }

    #[test]
    fn physical_allocation_keeps_the_original_signal_witness() {
        use worth_execution::ExecutionPhysicalAllocationDenialKind as Kind;
        use worth_foundational::{
            ExecutionPhysicalReport, ExecutionPosture, ExecutionReport, PartitionIdentity,
        };
        use worth_signal::facade::{
            SignalExecutionStop, SignalExecutionStopReason, SignalPublicationProgress,
        };
        let identity = PartitionIdentity::new(3);
        for cause in [
            Kind::Layout,
            Kind::Allocator,
            Kind::CapacityMismatch,
            Kind::WriteBeyondReserved,
            Kind::IncompleteSeal,
        ] {
            let original = SignalError::execution_stopped(SignalExecutionStop::new(
                SignalExecutionStopReason::Allocation(cause),
                Some(identity),
                SignalPublicationProgress::default(),
                ExecutionReport::new(
                    ExecutionPosture::Serial,
                    0,
                    0,
                    ExecutionPhysicalReport::default(),
                ),
            ));
            assert_eq!(
                kind(&original),
                super::Kind::SignalExecution(super::super::BridgeSignalDenial::Error(original))
            );
        }
    }
}
