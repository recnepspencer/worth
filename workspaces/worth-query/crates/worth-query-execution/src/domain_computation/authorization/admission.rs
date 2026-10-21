//! Conventional authorization admission facade.

use super::WorthQueryAuthorizationDecisionFact;
use super::WorthQueryPrincipalCurrentnessDependency;
use crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity;
use crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_query_installation::facade::WorthQueryInstalledAbilityRequirement;

mod operation_observation;
mod requirement_observation;
mod validation;

pub(super) use validation::admit_request;
pub(super) use validation::operation_scope_binding;
pub(super) use validation::validate_static_authority;
pub(in crate::domain_computation::authorization) use validation::validate_static_authority_retained;
pub(in crate::domain_computation::authorization) use validation::WorthQueryRetainedMutationStaticStop;

pub(in crate::domain_computation::authorization) struct WorthQueryConventionalAuthorizationDecisionPermit(
    (),
);

impl WorthQueryConventionalAuthorizationDecisionPermit {
    fn new() -> Self {
        Self(())
    }
}

struct WorthQueryConventionalAuthorizationObservation<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Scope,
> {
    session_identity:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    relational: &'a worth_relational::facade::runtime::RelationalRuntime,
    snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    principal: &'a WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    scope_identity: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
    binding_identity: &'a ApplicationSchemaBindingIdentity,
    requirements: &'a [WorthQueryInstalledAbilityRequirement],
}

struct WorthQueryObservedConventionalOperation {
    session_identity:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    principal_currentness: WorthQueryPrincipalCurrentnessDependency,
    decision_facts: Vec<WorthQueryAuthorizationDecisionFact>,
}
