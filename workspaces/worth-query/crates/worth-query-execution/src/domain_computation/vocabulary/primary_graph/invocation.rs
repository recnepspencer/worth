//! Moved whole from `domain_computation::primary_graph::vocabulary::application_query::invocation`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::provider_failure::WorthQueryManagedIndexProviderFailure;
use crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop;

/// A controlled request refusal or the provider's retained concrete domain cause.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexWorkDenial {
    Controlled(Stop),
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

impl From<Stop> for WorthQueryManagedIndexWorkDenial {
    fn from(cause: Stop) -> Self {
        Self::Controlled(cause)
    }
}
