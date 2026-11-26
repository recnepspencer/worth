//! Refusals to open a whole-result live subscription.
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationLiveControlDenial as Controls, WorthQueryApplicationLiveOpenDenial,
    WorthQueryApplicationLiveOpenDenialKind as Open,
};
use worth_query_publication::facade::application_entry::WorthQueryApplicationLiveOpenRequestDenial;

use super::query_read::admission;
use super::request::{BUSY, INTERNAL, PERMISSION, STORAGE, VERSION_MISMATCH};
use super::stops::authorization;
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

impl WorthQueryUserFacingRefusal for WorthQueryApplicationLiveOpenRequestDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::ExecutionRequest(cause) => cause.user_summary(),
            Self::InputPreparation(cause) => cause.user_summary(),
            Self::ScopeResolution(cause) => cause.user_summary(),
            Self::Open(cause) => live_open(cause),
            Self::PrincipalResolution(_) | Self::CapabilityAdmission(_) => PERMISSION,
            Self::Limit(_) => Summary::with_message(
                Category::Capacity,
                "There are too many results to show live. Narrow the view and try again.",
            ),
            Self::ProductSelection(_) => BUSY,
            Self::RetainedBasis
            | Self::ForeignRuntime
            | Self::BindingInstallation(_)
            | Self::CapabilityInstallation(_)
            | Self::Controls(Controls::Delivery(_) | Controls::ZeroMaterializedRecordLimit) => {
                INTERNAL
            }
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryApplicationLiveOpenDenial {
    fn user_summary(&self) -> Summary {
        live_open(self)
    }
}

pub(super) fn live_open(denial: &WorthQueryApplicationLiveOpenDenial) -> Summary {
    match denial.kind() {
        Open::Admission(kind) => admission(&kind),
        Open::AuthorizationDenied(kind) => authorization(&kind),
        Open::Content(_) => STORAGE,
        Open::BasisReleaseFailed | Open::ProviderVersionUnavailable | Open::BridgeBasisRejected => {
            BUSY
        }
        Open::LiveContractMissing | Open::BindingMismatch => VERSION_MISMATCH,
        Open::ScopeIdentityUnavailable | Open::ForeignAdvancementPhase => INTERNAL,
    }
}
