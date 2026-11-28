/// Typed pre-effect denials for product-reference retirement. Retirement
/// never cascades into a component-owner branch deletion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldBranchRetirementDenial {
    StaleIncarnation,
    OwnerUnavailable,
    AlreadyRetired,
    CapacityExhausted,
    Request(worth_execution::WorkCeilingDenial),
}

use worth_execution::report::{DenialClass, Disposition, ReportCode, Reportable, RetryHint};

impl std::fmt::Display for RuntimeWorldBranchRetirementDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.diagnostic_summary())
    }
}

impl std::error::Error for RuntimeWorldBranchRetirementDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Request(cause) => Some(cause),
            _ => None,
        }
    }
}

impl Reportable for RuntimeWorldBranchRetirementDenial {
    fn denial_class(&self) -> DenialClass {
        match self {
            Self::StaleIncarnation | Self::AlreadyRetired => DenialClass::Conflict,
            Self::OwnerUnavailable => DenialClass::Unavailable,
            Self::CapacityExhausted => DenialClass::Exhausted,
            Self::Request(cause) => cause.denial_class(),
        }
    }

    fn disposition(&self) -> Disposition {
        match self {
            Self::StaleIncarnation | Self::OwnerUnavailable => {
                Disposition::Retry(RetryHint::WithFreshAuthority)
            }
            Self::AlreadyRetired | Self::CapacityExhausted => Disposition::Final,
            Self::Request(cause) => cause.disposition(),
        }
    }

    fn report_code(&self) -> ReportCode {
        match self {
            Self::StaleIncarnation => ReportCode::new("world.branch_retirement.stale_incarnation")
                .expect("static report code"),
            Self::OwnerUnavailable => ReportCode::new("world.branch_retirement.owner_unavailable")
                .expect("static report code"),
            Self::AlreadyRetired => ReportCode::new("world.branch_retirement.already_retired")
                .expect("static report code"),
            Self::CapacityExhausted => {
                ReportCode::new("world.branch_retirement.capacity_exhausted")
                    .expect("static report code")
            }
            Self::Request(cause) => cause.report_code(),
        }
    }

    fn diagnostic_summary(&self) -> &'static str {
        match self {
            Self::StaleIncarnation => "The branch selection belongs to a stale incarnation.",
            Self::OwnerUnavailable => "The World owner is unavailable.",
            Self::AlreadyRetired => "The product branch is already retired.",
            Self::CapacityExhausted => "The World branch retirement capacity is exhausted.",
            Self::Request(_) => "The execution request did not complete branch retirement.",
        }
    }
}
