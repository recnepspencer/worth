use worth_execution::report::{DenialClass, Disposition, ReportCode, Reportable};

use super::{SignalRuntimePolicyAdmissionDenial, SignalRuntimePolicyCompilationDenial};

impl std::fmt::Display for SignalRuntimePolicyAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Signal runtime policy admission was denied.")
    }
}

impl std::error::Error for SignalRuntimePolicyAdmissionDenial {}

impl Reportable for SignalRuntimePolicyAdmissionDenial {
    fn denial_class(&self) -> DenialClass {
        DenialClass::InvalidInput
    }

    fn disposition(&self) -> Disposition {
        Disposition::CorrectInput
    }

    fn report_code(&self) -> ReportCode {
        ReportCode::new("signal.policy.admission_denied").expect("valid static report code")
    }

    fn diagnostic_summary(&self) -> &'static str {
        "Signal runtime policy admission was denied."
    }
}

impl std::fmt::Display for SignalRuntimePolicyCompilationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Signal runtime policy compilation failed.")
    }
}

impl std::error::Error for SignalRuntimePolicyCompilationDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Admission(denial) => Some(denial),
            Self::ObservationSessionActive => None,
        }
    }
}

impl Reportable for SignalRuntimePolicyCompilationDenial {
    fn denial_class(&self) -> DenialClass {
        match self {
            Self::Admission(denial) => denial.denial_class(),
            Self::ObservationSessionActive => DenialClass::Conflict,
        }
    }

    fn disposition(&self) -> Disposition {
        match self {
            Self::Admission(denial) => denial.disposition(),
            Self::ObservationSessionActive => Disposition::Final,
        }
    }

    fn report_code(&self) -> ReportCode {
        ReportCode::new("signal.policy.compilation_denied").expect("valid static report code")
    }

    fn diagnostic_summary(&self) -> &'static str {
        "Signal runtime policy compilation failed."
    }
}
