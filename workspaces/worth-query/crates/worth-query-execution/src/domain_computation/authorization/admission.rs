//! Conventional authorization admission facade.

use super::{WorthQueryAuthorizationDecisionFact, WorthQueryPrincipalCurrentnessDependency};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
};
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledAbilityRequirement,
};

mod operation_observation;
mod requirement_observation;
mod validation;

pub(super) use validation::{admit_request, operation_scope_binding, validate_static_authority};

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
