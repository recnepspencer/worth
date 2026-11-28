use core::{error::Error, fmt};

use super::{KernelStop, WorkCeilingDenial};
use crate::report::{DenialClass, Disposition, ReportCode, Reportable};
use crate::{LeaseDenial, MemoryLimitDenial};

impl Reportable for WorkCeilingDenial {
    fn denial_class(&self) -> DenialClass {
        match self {
            Self::Admission(
                LeaseDenial::WorkerLimitExceedsParent
                | LeaseDenial::MemoryLimitExceedsParent
                | LeaseDenial::WorkLimitExceedsParent
                | LeaseDenial::MemoryExhausted(_)
                | LeaseDenial::ChargedBytesOverflow,
            )
            | Self::Stopped(KernelStop::WorkCeiling | KernelStop::WorkCounterOverflow) => {
                DenialClass::Exhausted
            }
            Self::Admission(LeaseDenial::UnrelatedNestedLease) => DenialClass::Authorization,
            Self::Admission(
                LeaseDenial::NoActiveExecutionScope | LeaseDenial::EquivalenceContractUnavailable,
            )
            | Self::Stopped(KernelStop::NestedStopped | KernelStop::AccountingUnavailable) => {
                DenialClass::Unavailable
            }
            Self::Stopped(KernelStop::Cancelled) => DenialClass::Cancelled,
            Self::Stopped(KernelStop::DeadlineElapsed) => DenialClass::DeadlineElapsed,
            Self::Panicked => DenialClass::Defect,
        }
    }

    fn disposition(&self) -> Disposition {
        match self {
            Self::Panicked => Disposition::Escalate,
            Self::Admission(LeaseDenial::UnrelatedNestedLease) => Disposition::Reauthorize,
            _ => Disposition::Final,
        }
    }

    fn report_code(&self) -> ReportCode {
        let code = match self {
            Self::Admission(_) => "execution.work_ceiling.admission",
            Self::Stopped(_) => "execution.work_ceiling.stopped",
            Self::Panicked => "execution.work_ceiling.panicked",
        };
        ReportCode::new(code).expect("static report code")
    }

    fn diagnostic_summary(&self) -> &'static str {
        match self {
            Self::Admission(_) => "Execution refused to admit the computation.",
            Self::Stopped(_) => "The execution request stopped the computation.",
            Self::Panicked => "The computation panicked.",
        }
    }
}

impl fmt::Display for WorkCeilingDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.diagnostic_summary())
    }
}

impl Error for WorkCeilingDenial {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Admission(cause) => Some(cause),
            Self::Stopped(cause) => Some(cause),
            Self::Panicked => None,
        }
    }
}

impl fmt::Display for LeaseDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Execution lease admission refused.")
    }
}

impl Error for LeaseDenial {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MemoryExhausted(cause) => Some(cause),
            _ => None,
        }
    }
}

impl fmt::Display for MemoryLimitDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Execution memory limit refused the charge.")
    }
}

impl Error for MemoryLimitDenial {}

impl fmt::Display for KernelStop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Cancelled => "The execution request was cancelled.",
            Self::DeadlineElapsed => "The execution deadline elapsed.",
            Self::WorkCounterOverflow => "The execution work counter overflowed.",
            Self::WorkCeiling => "The execution work ceiling was exhausted.",
            Self::NestedStopped => "Nested execution stopped.",
            Self::AccountingUnavailable => "Execution work accounting is unavailable.",
        })
    }
}

impl Error for KernelStop {}
