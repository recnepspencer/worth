//! Managed-index controlled work stops and retained provider refusal vocabulary.

pub mod control;
pub mod provider_failure;
pub mod stop;

pub use control::WorthQueryManagedIndexControl;
pub use provider_failure::WorthQueryManagedIndexProviderFailure;
pub use stop::{WorthQueryManagedIndexProviderStop, WorthQueryManagedIndexStop};

/// A controlled request refusal or the provider's retained concrete domain cause.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexWorkDenial {
    Controlled(WorthQueryManagedIndexStop),
    Provider(WorthQueryManagedIndexProviderFailure),
}

impl worth_execution::ChargedBytes for WorthQueryManagedIndexWorkDenial {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Provider(cause) => cause.retained_payload_bytes(),
            Self::Controlled(_) => 0,
        }
    }
}

impl std::fmt::Display for WorthQueryManagedIndexWorkDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for WorthQueryManagedIndexWorkDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Provider(cause) => Some(cause),
            Self::Controlled(_) => None,
        }
    }
}

impl From<WorthQueryManagedIndexStop> for WorthQueryManagedIndexWorkDenial {
    fn from(cause: WorthQueryManagedIndexStop) -> Self {
        Self::Controlled(cause)
    }
}
