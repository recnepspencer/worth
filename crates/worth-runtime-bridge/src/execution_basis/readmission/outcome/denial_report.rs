use super::BridgeExecutionBasisReadmissionDenialKind;
use super::BridgeExecutionBasisReadmissionDenied;
use worth_execution::report::{
    DenialClass::{self, *},
    Disposition::{self, *},
    ReportCode, Reportable,
};

impl std::fmt::Debug for BridgeExecutionBasisReadmissionDenied {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeExecutionBasisReadmissionDenied")
            .field("kind", &self.kind())
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for BridgeExecutionBasisReadmissionDenied {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Bridge execution-basis readmission was denied.")
    }
}

impl std::error::Error for BridgeExecutionBasisReadmissionDenied {}

impl Reportable for BridgeExecutionBasisReadmissionDenied {
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
        "Bridge execution-basis readmission was denied."
    }
}

fn report(
    denial: &BridgeExecutionBasisReadmissionDenied,
) -> (DenialClass, Disposition, &'static str) {
    match denial.kind() {
        BridgeExecutionBasisReadmissionDenialKind::ForeignRuntime => (
            Authorization,
            Reauthorize,
            "bridge.readmission.foreign_runtime",
        ),
        BridgeExecutionBasisReadmissionDenialKind::SourceProfileMismatch => (
            InvalidInput,
            CorrectInput,
            "bridge.readmission.source_profile_mismatch",
        ),
        BridgeExecutionBasisReadmissionDenialKind::OperationBindingMismatch => (
            Authorization,
            Reauthorize,
            "bridge.readmission.operation_binding_mismatch",
        ),
        BridgeExecutionBasisReadmissionDenialKind::AttemptIdentityReused => (
            Conflict,
            Final,
            "bridge.readmission.attempt_identity_reused",
        ),
        BridgeExecutionBasisReadmissionDenialKind::InvalidManagedExecutionIntent => (
            InvalidInput,
            CorrectInput,
            "bridge.readmission.invalid_managed_execution_intent",
        ),
        BridgeExecutionBasisReadmissionDenialKind::ManagedExecutionIntentAlreadyReserved => (
            Conflict,
            Final,
            "bridge.readmission.managed_execution_intent_already_reserved",
        ),
        BridgeExecutionBasisReadmissionDenialKind::SignalDeclarationUnavailable => (
            Unavailable,
            Escalate,
            "bridge.readmission.signal_declaration_unavailable",
        ),
        BridgeExecutionBasisReadmissionDenialKind::SignalAttemptAdmissionFailed => (
            Unavailable,
            Escalate,
            "bridge.readmission.signal_attempt_admission_failed",
        ),
        BridgeExecutionBasisReadmissionDenialKind::SignalAttemptMissing => (
            NotFound,
            CorrectInput,
            "bridge.readmission.signal_attempt_missing",
        ),
        BridgeExecutionBasisReadmissionDenialKind::SignalAttemptMismatch => (
            Invariant,
            Escalate,
            "bridge.readmission.signal_attempt_mismatch",
        ),
        BridgeExecutionBasisReadmissionDenialKind::SignalManagedQueueBindingFailed => (
            Unavailable,
            Escalate,
            "bridge.readmission.signal_managed_queue_binding_failed",
        ),
        BridgeExecutionBasisReadmissionDenialKind::YieldPostureMismatch => {
            (Conflict, Final, "bridge.readmission.yield_posture_mismatch")
        }
    }
}
