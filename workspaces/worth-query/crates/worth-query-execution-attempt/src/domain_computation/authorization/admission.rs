//! Conventional authorization admission facade.

use super::WorthQueryAuthorizationDecisionFact;
use super::WorthQueryPrincipalCurrentnessDependency;
use crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity;
use crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_query_installation::facade::WorthQueryInstalledAbilityRequirement;

pub struct WorthQueryConventionalAuthorizationDecisionPermit(());

impl WorthQueryConventionalAuthorizationDecisionPermit {
    pub fn new() -> Self {
        Self(())
    }
}

pub struct WorthQueryConventionalAuthorizationObservation<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Scope,
> {
    pub session_identity:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    pub relational: &'a worth_relational::facade::runtime::RelationalRuntime,
    pub snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    pub principal: &'a WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    pub scope_identity: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
    pub binding_identity: &'a ApplicationSchemaBindingIdentity,
    pub requirements: &'a [WorthQueryInstalledAbilityRequirement],
}

pub struct WorthQueryObservedConventionalOperation {
    pub session_identity:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    pub principal_currentness: WorthQueryPrincipalCurrentnessDependency,
    pub decision_facts: Vec<WorthQueryAuthorizationDecisionFact>,
}
