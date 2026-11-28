use std::{error::Error, fmt};

use worth_execution::report::{DenialClass, Disposition, ReportCode, Reportable, RetryHint};

use super::RuntimeWorldOwnerUnavailable;

/// Separates weak-owner loss from an available service's domain denial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeWorldServiceDenial<D> {
    OwnerUnavailable(super::RuntimeWorldOwnerUnavailable),
    Denied(D),
}
impl<D> From<super::RuntimeWorldOwnerUnavailable> for RuntimeWorldServiceDenial<D> {
    fn from(value: super::RuntimeWorldOwnerUnavailable) -> Self {
        Self::OwnerUnavailable(value)
    }
}

impl<D: fmt::Display> fmt::Display for RuntimeWorldServiceDenial<D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnerUnavailable(cause) => cause.fmt(formatter),
            Self::Denied(cause) => cause.fmt(formatter),
        }
    }
}

impl<D: Error + 'static> Error for RuntimeWorldServiceDenial<D> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OwnerUnavailable(cause) => Some(cause),
            Self::Denied(cause) => Some(cause),
        }
    }
}

impl<D: Reportable> RuntimeWorldServiceDenial<D> {
    fn report_cause(&self) -> &dyn Reportable {
        match self {
            Self::OwnerUnavailable(cause) => cause,
            Self::Denied(cause) => cause,
        }
    }
}

impl<D: Reportable> Reportable for RuntimeWorldServiceDenial<D> {
    fn denial_class(&self) -> DenialClass {
        self.report_cause().denial_class()
    }

    fn disposition(&self) -> Disposition {
        self.report_cause().disposition()
    }

    fn report_code(&self) -> ReportCode {
        self.report_cause().report_code()
    }

    fn diagnostic_summary(&self) -> &'static str {
        self.report_cause().diagnostic_summary()
    }
}

crate::denial_report::world_denial_report!(RuntimeWorldOwnerUnavailable, "service", {
    Self { .. } => (
        "owner_unavailable",
        Unavailable,
        Disposition::Retry(RetryHint::WithFreshAuthority),
        "The World owner is unavailable."
    ),
});

impl Error for RuntimeWorldOwnerUnavailable {}

#[cfg(test)]
mod tests;
