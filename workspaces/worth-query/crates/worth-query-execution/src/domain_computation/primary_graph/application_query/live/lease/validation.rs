use worth_query_declaration::facade::application_query::ApplicationQueryLiveCauseBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::super::{
    controls::WorthQueryApplicationLiveControls,
    outcome::{WorthQueryApplicationLiveOpenDenial, WorthQueryApplicationLiveOpenDenialKind},
};
use crate::domain_computation::primary_graph::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;

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

pub(super) fn validate_live_resource_controls(
    live: &worth_query_installation::facade::WorthQueryInstalledApplicationLiveContract,
    controls: &WorthQueryApplicationLiveControls,
    subject: &str,
) -> Result<(), WorthQueryApplicationLiveOpenDenial> {
    if controls.buffer_capacity() as u64 > live.resource_envelope().queue_depth_ceiling() {
        return Err(open_denial(
            WorthQueryApplicationLiveOpenDenialKind::BufferCapacityExceedsInstalled,
            subject,
        ));
    }
    let installed_work = live
        .resource_envelope()
        .bounded_step_contract()
        .map_err(|detail| {
            open_denial(
                WorthQueryApplicationLiveOpenDenialKind::BridgeBasisRejected,
                detail,
            )
        })?
        .max_work_units_per_step();
    if controls.maximum_work_per_delivery().get() as u64 > installed_work {
        return Err(open_denial(
            WorthQueryApplicationLiveOpenDenialKind::WorkLimitExceedsInstalled,
            subject,
        ));
    }
    Ok(())
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
                super::super::super::WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal,
            ),
            subject,
        ),
        WorthQueryAuthorizedApplicationReadDenial::StaleScope
        | WorthQueryAuthorizedApplicationReadDenial::StaleBasisScope => open_denial(
            WorthQueryApplicationLiveOpenDenialKind::Admission(
                super::super::super::WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
            ),
            subject,
        ),
        WorthQueryAuthorizedApplicationReadDenial::Authorization(authorization) => {
            let kind =
                WorthQueryApplicationLiveOpenDenialKind::AuthorizationDenied(authorization.kind());
            WorthQueryApplicationLiveOpenDenial::with_authorization(kind, authorization)
        }
        WorthQueryAuthorizedApplicationReadDenial::Read(_)
        | WorthQueryAuthorizedApplicationReadDenial::Session => open_denial(
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
