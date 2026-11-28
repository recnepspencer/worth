use super::SignalBranchRetirementDenial;
use worth_execution::report::{DenialClass, Disposition, ReportCode, Reportable};

impl std::fmt::Display for SignalBranchRetirementDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Signal branch retirement was denied.")
    }
}

impl std::error::Error for SignalBranchRetirementDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OwnerUnavailable(denial) => Some(denial),
            _ => None,
        }
    }
}

impl Reportable for SignalBranchRetirementDenial {
    fn denial_class(&self) -> DenialClass {
        match self {
            Self::OwnerUnavailable(_) | Self::QuarantinedBranch { .. } => DenialClass::Unavailable,
            Self::OperationCapacityExhausted { .. }
            | Self::RetirementReceiptCapacityExhausted { .. } => DenialClass::Exhausted,
            Self::CancelledNoMovement => DenialClass::Cancelled,
            Self::UnknownBranch { .. } | Self::RetiredBranch { .. } => DenialClass::NotFound,
            Self::OwnerReentry
            | Self::RetirementInProgress { .. }
            | Self::CurrentBranch { .. }
            | Self::CanonicalBranch { .. }
            | Self::StaleBranchHead { .. }
            | Self::LiveChildren { .. }
            | Self::RetainedComponentBasis { .. }
            | Self::RetainedAdmittedBasis { .. }
            | Self::SharedAdmittedBasis { .. } => DenialClass::Conflict,
            Self::CanonicalBasisMismatch
            | Self::ForeignRetirementSnapshot { .. }
            | Self::RetirementSnapshotBranchMismatch { .. } => DenialClass::Authorization,
            Self::OwnerCellMisuse { .. } | Self::OwnerInvariantViolation { .. } => {
                DenialClass::Defect
            }
        }
    }

    fn disposition(&self) -> Disposition {
        match self.denial_class() {
            DenialClass::Authorization => Disposition::Reauthorize,
            DenialClass::NotFound => Disposition::CorrectInput,
            DenialClass::Cancelled | DenialClass::Conflict => Disposition::Final,
            _ => Disposition::Escalate,
        }
    }

    fn report_code(&self) -> ReportCode {
        ReportCode::new("signal.branch.retirement_denied").expect("valid static report code")
    }

    fn diagnostic_summary(&self) -> &'static str {
        "Signal branch retirement was denied."
    }
}
