use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome,
    primary_graph::{
        WorthQueryApplicationLiveCauseDenialKind,
        WorthQueryApplicationLiveOutcome, WorthQueryApplicationLiveOverflow,
        WorthQueryApplicationProjectionDenialKind, WorthQueryOperationAuthorizationDenial,
    },
};

use crate::{
    declaration::{WorthUiStatusQuery, WorthUiStatusUpdateDenial},
    WorthUiScalarProjectionSourceRecord, WorthUiStatusActionRequest, WorthUiStatusQueryResult,
};

pub type WorthUiStatusMutationOutcome = WorthQueryApplicationMutationOutcome<
    WorthUiStatusUpdateDenial,
    WorthUiScalarProjectionSourceRecord,
>;
pub type WorthUiStatusActionMutationOutcome =
    WorthQueryApplicationMutationOutcome<WorthUiStatusUpdateDenial, WorthUiStatusActionRequest>;

#[derive(Debug)]
pub enum WorthUiStatusLiveDeliveryStop {
    Pending,
    Overflow(WorthQueryApplicationLiveOverflow),
    AuthorizationDenied(Box<WorthQueryOperationAuthorizationDenial>),
    StalePrincipal,
    StaleScope,
    ProjectionDenied(WorthQueryApplicationProjectionDenialKind),
    CauseDenied(WorthQueryApplicationLiveCauseDenialKind),
    ExecutionRequest(worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind),
    Cancelled,
    DeadlineExceeded,
    Closed,
    Unavailable,
}

/// Setup/read diagnostics carry no mutation custody. Ordinary mutation stops return
/// the actual `WorthUiStatusMutation` continuation separately.
#[derive(Debug)]
pub enum WorthUiStatusOwnerError {
    Installation(String),
    AuthenticationAdmission(worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticationAdapterAdmissionDenial),
    Authentication(worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticationDenial),
    Query(String),
    MissingUniqueSource,
    MissingUniqueRecord,
    LiveOpen(String),
    LiveDelivery(WorthUiStatusLiveDeliveryStop),
}

impl std::fmt::Display for WorthUiStatusOwnerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

pub(super) fn classify_live_delivery(
    outcome: WorthQueryApplicationLiveOutcome<WorthUiStatusQuery, WorthUiStatusQueryResult>,
) -> Result<
    worth_query_host::facade::primary_graph::WorthQueryApplicationLiveUpdate<
        WorthUiStatusQuery,
        WorthUiStatusQueryResult,
    >,
    WorthUiStatusOwnerError,
> {
    use WorthQueryApplicationLiveOutcome as Outcome;
    let stop = match outcome {
        Outcome::Delivered(update) => return Ok(update),
        Outcome::Pending => WorthUiStatusLiveDeliveryStop::Pending,
        Outcome::Overflow(overflow) => WorthUiStatusLiveDeliveryStop::Overflow(overflow),
        Outcome::AuthorizationDenied(denial) => {
            WorthUiStatusLiveDeliveryStop::AuthorizationDenied(denial)
        }
        Outcome::StalePrincipal => WorthUiStatusLiveDeliveryStop::StalePrincipal,
        Outcome::StaleScope => WorthUiStatusLiveDeliveryStop::StaleScope,
        Outcome::ProjectionDenied(denial) => {
            WorthUiStatusLiveDeliveryStop::ProjectionDenied(denial)
        }
        Outcome::CauseDenied(denial) => WorthUiStatusLiveDeliveryStop::CauseDenied(denial),
        Outcome::ExecutionRequest(cause) => WorthUiStatusLiveDeliveryStop::ExecutionRequest(cause),
        Outcome::Cancelled => WorthUiStatusLiveDeliveryStop::Cancelled,
        Outcome::DeadlineExceeded => WorthUiStatusLiveDeliveryStop::DeadlineExceeded,
        Outcome::Closed => WorthUiStatusLiveDeliveryStop::Closed,
        Outcome::Unavailable => WorthUiStatusLiveDeliveryStop::Unavailable,
    };
    Err(WorthUiStatusOwnerError::LiveDelivery(stop))
}

#[cfg(test)]
mod execution_delivery_tests {
    use super::*;
    #[test]
    fn ui_live_delivery_carries_the_query_execution_cause_by_value() {
        use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
        use worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind as Kind;
        let cause = Kind::ExecutionResource {
            denial: Resource::WorkExhausted,
            partition_identity: Some(3),
            policy_ancestor: Some(2),
        };
        let answer =
            classify_live_delivery(WorthQueryApplicationLiveOutcome::ExecutionRequest(cause));
        let Err(WorthUiStatusOwnerError::LiveDelivery(
            WorthUiStatusLiveDeliveryStop::ExecutionRequest(actual),
        )) = answer
        else {
            panic!("the UI adapter keeps the typed query stop")
        };
        assert_eq!(actual, cause);
    }
}
