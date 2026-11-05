use worth_query_declaration::facade::application_query::ApplicationQueryLiveCauseBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::super::outcome::{
    WorthQueryApplicationLiveOpenDenial, WorthQueryApplicationLiveOpenDenialKind,
};
use crate::domain_computation::primary_graph::vocabulary::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial;

pub(super) fn validate_live_binding<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
    Target,
    Binding,
>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
) -> Result<
    &worth_query_installation::facade::WorthQueryInstalledApplicationLiveContract,
    WorthQueryApplicationLiveOpenDenial,
>
where
    Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
{
    let live = query.live().ok_or_else(|| {
        open_denial(
            WorthQueryApplicationLiveOpenDenialKind::LiveContractMissing,
            query.name(),
        )
    })?;
    let binding_matches = live.binding_type()
        == <Binding as worth_query_declaration::facade::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY.as_str()
        && live.effect() == Binding::effect().name()
        && live.payload_type()
            == <Binding::PayloadBinding as ApplicationStructuredValueBinding>::IDENTITY.as_str();
    if binding_matches {
        Ok(live)
    } else {
        Err(open_denial(
            WorthQueryApplicationLiveOpenDenialKind::BindingMismatch,
            query.name(),
        ))
    }
}

pub(super) fn open_admission_denial(
    denial: WorthQueryApplicationQueryAdmissionDenial,
) -> WorthQueryApplicationLiveOpenDenial {
    let kind = WorthQueryApplicationLiveOpenDenialKind::Admission(denial.kind());
    let subject = denial.subject().to_string();
    match denial.into_authorization_denial() {
        Some(authorization) => {
            WorthQueryApplicationLiveOpenDenial::with_authorization(kind, authorization)
        }
        None => open_denial(kind, subject),
    }
}

pub(super) fn open_read_denial(
    denial: WorthQueryAuthorizedApplicationReadDenial,
    subject: &str,
) -> WorthQueryApplicationLiveOpenDenial {
    match denial {
        WorthQueryAuthorizedApplicationReadDenial::StalePrincipal => open_denial(
            WorthQueryApplicationLiveOpenDenialKind::Admission(
                crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal,
            ),
            subject,
        ),
        WorthQueryAuthorizedApplicationReadDenial::StaleScope
        | WorthQueryAuthorizedApplicationReadDenial::StaleBasisScope => open_denial(
            WorthQueryApplicationLiveOpenDenialKind::Admission(
                crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
            ),
            subject,
        ),
        WorthQueryAuthorizedApplicationReadDenial::Authorization(authorization) => {
            let kind =
                WorthQueryApplicationLiveOpenDenialKind::AuthorizationDenied(authorization.kind());
            WorthQueryApplicationLiveOpenDenial::with_authorization(kind, authorization)
        }
        WorthQueryAuthorizedApplicationReadDenial::Read(read) => {
            use crate::domain_computation::primary_graph::vocabulary::application_query::read_execution_denial::WorthQueryApplicationReadExecutionDenialKind as Read;
            use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind as Admission;
            let kind = match read.kind() {
                Read::Content(cause) => WorthQueryApplicationLiveOpenDenialKind::Content(cause),
                Read::Cancelled => {
                    WorthQueryApplicationLiveOpenDenialKind::Admission(Admission::Cancelled)
                }
                Read::DeadlineExceeded => {
                    WorthQueryApplicationLiveOpenDenialKind::Admission(Admission::DeadlineExceeded)
                }
                _ => WorthQueryApplicationLiveOpenDenialKind::ScopeIdentityUnavailable,
            };
            open_denial(kind, read.subject())
        }
        WorthQueryAuthorizedApplicationReadDenial::Session => open_denial(
            WorthQueryApplicationLiveOpenDenialKind::ScopeIdentityUnavailable,
            subject,
        ),
    }
}

pub(super) fn open_denial(
    kind: WorthQueryApplicationLiveOpenDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationLiveOpenDenial {
    WorthQueryApplicationLiveOpenDenial::new(kind, subject)
}

pub(super) fn subscription_capacity_denial(subject: &str) -> WorthQueryApplicationLiveOpenDenial {
    use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind as Admission;
    open_denial(
        WorthQueryApplicationLiveOpenDenialKind::Admission(Admission::RetentionCapacityExhausted),
        subject,
    )
}
