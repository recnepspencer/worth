use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::super::{
    admission_preparation::validate_admission_request,
    disclosure::{
        WorthQueryApplicationQueryGovernanceDenialKind, WorthQueryPendingApplicationQueryGovernance,
    },
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(in crate::domain_computation::primary_graph::application_query) fn prepare_governed_access<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Capability,
    Operation,
    Input,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    access: &WorthQueryApplicationQueryAccessContext<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
        Schema,
        Capability,
        Operation,
        Input,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<WorthQueryPendingApplicationQueryGovernance, WorthQueryApplicationQueryAdmissionDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
{
    validate_admission_request(capability.request_scope(), query.name())?;
    if capability.runtime_authority() != runtime.runtime.authority_identity()
        || capability.binding_identity() != query.binding_identity()
        || capability.principal_entity_id() != access.principal().principal_entity_id()
        || capability.resource_entity_id() != access.scope().entity_id()
        || runtime.authentication_is_expired(capability.authentication_valid_until())
    {
        return Err(denial(
            WorthQueryApplicationQueryAdmissionDenialKind::DisclosureAuthorizationMismatch,
            query.name(),
        ));
    }
    validate_admission_request(request, query.name())?;
    let capability_name = capability.capability_name().to_string();
    let capability_type = capability.capability_type().to_string();
    let authorization_canonical_work = capability.admission_canonical_work();
    let disclosure_value = capability.disclosure_value().cloned().ok_or_else(|| {
        denial(
            WorthQueryApplicationQueryAdmissionDenialKind::DisclosureAuthorizationMismatch,
            query.name(),
        )
    })?;
    Ok(WorthQueryPendingApplicationQueryGovernance::new(
        capability_name,
        capability_type,
        disclosure_value,
        authorization_canonical_work,
        capability.into_query_authorization(),
    ))
}

pub(in crate::domain_computation::primary_graph::application_query) fn governance_denial(
    kind: WorthQueryApplicationQueryGovernanceDenialKind,
    subject: &str,
) -> WorthQueryApplicationQueryAdmissionDenial {
    let kind = match kind {
        WorthQueryApplicationQueryGovernanceDenialKind::Required => {
            WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired
        }
        WorthQueryApplicationQueryGovernanceDenialKind::CapabilityMismatch => {
            WorthQueryApplicationQueryAdmissionDenialKind::DisclosureAuthorizationMismatch
        }
        WorthQueryApplicationQueryGovernanceDenialKind::InternalComputationDenied => {
            WorthQueryApplicationQueryAdmissionDenialKind::InternalComputationDenied
        }
    };
    denial(kind, subject)
}

fn denial(
    kind: WorthQueryApplicationQueryAdmissionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(kind, subject)
}
