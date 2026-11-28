use worth_execution::report::{DenialClass, Disposition, ReportCode, Reportable};

use super::{
    SignalCheckpointDenial, SignalError, SignalExecutionFailure, SignalExecutionStopReason,
    SignalLeaseDenial,
};

impl Reportable for SignalError {
    fn denial_class(&self) -> DenialClass {
        match self {
            Self::ExecutionStopped(stop) => match stop.reason() {
                SignalExecutionStopReason::Failure { cause, .. } => execution_failure_class(cause),
                SignalExecutionStopReason::Admission(denial) => admission_class(denial),
                SignalExecutionStopReason::WorkExhausted { .. }
                | SignalExecutionStopReason::Allocation(_)
                | SignalExecutionStopReason::PreparationMemoryExhausted { .. } => {
                    DenialClass::Exhausted
                }
            },
            Self::ExecutionAdmissionDenied(denial) => admission_class(denial),
            Self::ExecutionCheckpointStopped(denial) => checkpoint_class(*denial),
            Self::ExecutionPanicked
            | Self::RetainedStorageChargeOverflow
            | Self::RetainedStorageChargeUnderflow
            | Self::ScratchMismatch { .. }
            | Self::Internal { .. } => DenialClass::Defect,
            Self::MissingBoundedInputsAtEpochAdmission { .. }
            | Self::MissingBoundedInputsAtEvaluation { .. }
            | Self::IncompatibleSnapshot { .. }
            | Self::InvalidInput { .. } => DenialClass::InvalidInput,
            Self::ContractViolation { .. } | Self::TransactionPoisoned => DenialClass::Invariant,
            Self::PreparationMemoryExhausted { .. }
            | Self::EvaluationStorageCapacityExhausted
            | Self::CheckedResultCapacityExceeded { .. }
            | Self::ConditionalEvaluationWorkExhausted { .. }
            | Self::UpstreamDependencyWorkExhausted { .. }
            | Self::WaiterResolutionWorkExhausted { .. } => DenialClass::Exhausted,
            Self::EvaluationStorageUnavailable
            | Self::SnapshotIndexUnavailable
            | Self::RetainedStorageHistoryUnavailable => DenialClass::Unavailable,
            Self::StaleHandle { .. } | Self::UnknownBranch { .. } => DenialClass::NotFound,
            Self::CycleDetected { .. } => DenialClass::RelationLaw,
            Self::ScratchReentry { .. }
            | Self::TransactionFinished
            | Self::ManagedQueueBranchTransferDenied { .. } => DenialClass::Conflict,
            Self::EventFlushFailed { .. } => DenialClass::ExternalEffect,
        }
    }

    fn disposition(&self) -> Disposition {
        match self.denial_class() {
            DenialClass::InvalidInput | DenialClass::NotFound | DenialClass::RelationLaw => {
                Disposition::CorrectInput
            }
            DenialClass::Authorization => Disposition::Reauthorize,
            DenialClass::Cancelled | DenialClass::DeadlineElapsed | DenialClass::Conflict => {
                Disposition::Final
            }
            _ => Disposition::Escalate,
        }
    }

    fn report_code(&self) -> ReportCode {
        let code = match self {
            Self::ExecutionPanicked => "signal.execution_panicked",
            Self::RetainedStorageChargeOverflow => "signal.retained_storage_charge_overflow",
            Self::RetainedStorageChargeUnderflow => "signal.retained_storage_charge_underflow",
            Self::ScratchMismatch { .. } => "signal.scratch_mismatch",
            Self::Internal { .. } => "signal.internal",
            Self::MissingBoundedInputsAtEpochAdmission { .. } => {
                "signal.missing_bounded_inputs_at_epoch_admission"
            }
            Self::MissingBoundedInputsAtEvaluation { .. } => {
                "signal.missing_bounded_inputs_at_evaluation"
            }
            Self::IncompatibleSnapshot { .. } => "signal.incompatible_snapshot",
            Self::InvalidInput { .. } => "signal.invalid_input",
            Self::ContractViolation { .. } => "signal.contract_violation",
            Self::TransactionPoisoned => "signal.transaction_poisoned",
            Self::PreparationMemoryExhausted { .. } => "signal.preparation_memory_exhausted",
            Self::EvaluationStorageCapacityExhausted => {
                "signal.evaluation_storage_capacity_exhausted"
            }
            Self::CheckedResultCapacityExceeded { .. } => "signal.checked_result_capacity_exceeded",
            Self::ConditionalEvaluationWorkExhausted { .. } => {
                "signal.conditional_evaluation_work_exhausted"
            }
            Self::UpstreamDependencyWorkExhausted { .. } => {
                "signal.upstream_dependency_work_exhausted"
            }
            Self::WaiterResolutionWorkExhausted { .. } => "signal.waiter_resolution_work_exhausted",
            Self::EvaluationStorageUnavailable => "signal.evaluation_storage_unavailable",
            Self::SnapshotIndexUnavailable => "signal.snapshot_index_unavailable",
            Self::RetainedStorageHistoryUnavailable => {
                "signal.retained_storage_history_unavailable"
            }
            Self::StaleHandle { .. } => "signal.stale_handle",
            Self::UnknownBranch { .. } => "signal.unknown_branch",
            Self::CycleDetected { .. } => "signal.cycle_detected",
            Self::ScratchReentry { .. } => "signal.scratch_reentry",
            Self::TransactionFinished => "signal.transaction_finished",
            Self::ManagedQueueBranchTransferDenied { .. } => {
                "signal.managed_queue_branch_transfer_denied"
            }
            Self::EventFlushFailed { .. } => "signal.event_flush_failed",
            Self::ExecutionStopped(_) => "signal.execution_stopped",
            Self::ExecutionAdmissionDenied(_) => "signal.execution_admission_denied",
            Self::ExecutionCheckpointStopped(_) => "signal.execution_checkpoint_stopped",
        };
        ReportCode::new(code).expect("valid static Signal report code")
    }

    fn diagnostic_summary(&self) -> &'static str {
        "Signal could not complete the operation."
    }
}

fn admission_class(denial: &SignalLeaseDenial) -> DenialClass {
    match denial {
        SignalLeaseDenial::WorkerLimitExceedsParent
        | SignalLeaseDenial::MemoryLimitExceedsParent
        | SignalLeaseDenial::WorkLimitExceedsParent => DenialClass::InvalidInput,
        SignalLeaseDenial::MemoryExhausted(_) => DenialClass::Exhausted,
        SignalLeaseDenial::UnrelatedNestedLease => DenialClass::Authorization,
        SignalLeaseDenial::ChargedBytesOverflow => DenialClass::Defect,
        SignalLeaseDenial::NoActiveExecutionScope
        | SignalLeaseDenial::EquivalenceContractUnavailable => DenialClass::Unavailable,
    }
}

fn checkpoint_class(denial: SignalCheckpointDenial) -> DenialClass {
    match denial {
        SignalCheckpointDenial::Cancelled => DenialClass::Cancelled,
        SignalCheckpointDenial::DeadlineElapsed => DenialClass::DeadlineElapsed,
        SignalCheckpointDenial::WorkCeiling => DenialClass::Exhausted,
        SignalCheckpointDenial::WorkCounterOverflow => DenialClass::Defect,
        SignalCheckpointDenial::AccountingUnavailable => DenialClass::Unavailable,
        SignalCheckpointDenial::NestedStopped => DenialClass::Indeterminate,
    }
}

fn execution_failure_class(failure: &SignalExecutionFailure) -> DenialClass {
    match failure {
        SignalExecutionFailure::Domain(error) => error.denial_class(),
        SignalExecutionFailure::Cancelled => DenialClass::Cancelled,
        SignalExecutionFailure::DeadlineElapsed => DenialClass::DeadlineElapsed,
        SignalExecutionFailure::WorkCounterOverflow | SignalExecutionFailure::Panic => {
            DenialClass::Defect
        }
        SignalExecutionFailure::AccountingUnavailable => DenialClass::Unavailable,
        SignalExecutionFailure::WorkCeiling | SignalExecutionFailure::ResultCapacityExceeded => {
            DenialClass::Exhausted
        }
        SignalExecutionFailure::NestedStopped => DenialClass::Indeterminate,
    }
}

impl std::fmt::Display for SignalLeaseDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Execution admission denied.")
    }
}

impl std::error::Error for SignalLeaseDenial {}

impl std::fmt::Display for SignalCheckpointDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Execution checkpoint stopped.")
    }
}

impl std::error::Error for SignalCheckpointDenial {}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_execution::report::{DiagnosticDetail, ReportProjection};

    #[test]
    fn execution_admission_reports_keep_the_cause_and_exclude_occurrence_text() {
        let admission =
            SignalError::ExecutionAdmissionDenied(SignalLeaseDenial::UnrelatedNestedLease);
        let report = admission.to_operator_report(DiagnosticDetail::Standard);
        assert_eq!(report.denial_class(), DenialClass::Authorization);
        assert_eq!(report.disposition(), Disposition::Reauthorize);
        assert_eq!(
            report.report_code().as_str(),
            "signal.execution_admission_denied"
        );
        assert_eq!(report.chain().len(), 1);

        let first = SignalError::invalid_input("private-first")
            .to_operator_report(DiagnosticDetail::Forensic);
        let second = SignalError::invalid_input("private-second")
            .to_operator_report(DiagnosticDetail::Forensic);
        assert_eq!(first, second);
        assert_eq!(first.to_canonical_json(), second.to_canonical_json());
    }
}
