use super::BridgeExecutionDenial;
use worth_execution::report::{
    DenialClass::{self, *},
    Disposition::{self, *},
    ReportCode, Reportable,
};

impl std::fmt::Display for BridgeExecutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Bridge execution was stopped.")
    }
}

impl std::error::Error for BridgeExecutionDenial {}

impl Reportable for BridgeExecutionDenial {
    fn denial_class(&self) -> DenialClass {
        report(self).0
    }

    fn disposition(&self) -> Disposition {
        report(self).1
    }

    fn report_code(&self) -> ReportCode {
        ReportCode::new(report(self).2).expect("valid static Bridge report code")
    }

    fn diagnostic_summary(&self) -> &'static str {
        "Bridge execution was stopped."
    }
}

fn report(denial: &BridgeExecutionDenial) -> (DenialClass, Disposition, &'static str) {
    match denial {
        BridgeExecutionDenial::WorkerLimitExceedsParent => (
            InvalidInput,
            CorrectInput,
            "bridge.execution.worker_limit_exceeds_parent",
        ),
        BridgeExecutionDenial::MemoryLimitExceedsParent => (
            InvalidInput,
            CorrectInput,
            "bridge.execution.memory_limit_exceeds_parent",
        ),
        BridgeExecutionDenial::WorkLimitExceedsParent => (
            InvalidInput,
            CorrectInput,
            "bridge.execution.work_limit_exceeds_parent",
        ),
        BridgeExecutionDenial::MemoryExhausted(_) => {
            (Exhausted, Escalate, "bridge.execution.memory_exhausted")
        }
        BridgeExecutionDenial::ChargedBytesOverflow => {
            (Defect, Escalate, "bridge.execution.charged_bytes_overflow")
        }
        BridgeExecutionDenial::UnrelatedNestedLease => (
            Authorization,
            Reauthorize,
            "bridge.execution.unrelated_nested_lease",
        ),
        BridgeExecutionDenial::NoActiveExecutionScope => (
            Unavailable,
            Escalate,
            "bridge.execution.no_active_execution_scope",
        ),
        BridgeExecutionDenial::EquivalenceContractUnavailable => (
            Unavailable,
            Escalate,
            "bridge.execution.equivalence_contract_unavailable",
        ),
        BridgeExecutionDenial::Cancelled => (Cancelled, Final, "bridge.execution.cancelled"),
        BridgeExecutionDenial::DeadlineElapsed => {
            (DeadlineElapsed, Final, "bridge.execution.deadline_elapsed")
        }
        BridgeExecutionDenial::WorkCounterOverflow => {
            (Defect, Escalate, "bridge.execution.work_counter_overflow")
        }
        BridgeExecutionDenial::AccountingUnavailable => (
            Unavailable,
            Escalate,
            "bridge.execution.accounting_unavailable",
        ),
        BridgeExecutionDenial::WorkCeiling => {
            (Exhausted, Escalate, "bridge.execution.work_ceiling")
        }
        BridgeExecutionDenial::NestedStopped => {
            (Indeterminate, Escalate, "bridge.execution.nested_stopped")
        }
        BridgeExecutionDenial::Panicked => (Defect, Escalate, "bridge.execution.panicked"),
    }
}
