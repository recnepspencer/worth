use std::sync::Arc;

use crate::domain_computation::primary_graph::{
    WorthQueryAuthenticatedPrincipal, WorthQueryPrimaryPrincipalBindingLayout,
    WorthQueryPrincipalFreshnessEvidence,
};

#[derive(Clone)]
pub(in crate::domain_computation) struct WorthQueryPrincipalCurrentnessDependency {
    session_identity:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    binding: Arc<str>,
    layout: WorthQueryPrimaryPrincipalBindingLayout,
    freshness: WorthQueryPrincipalFreshnessEvidence,
}

impl WorthQueryPrincipalCurrentnessDependency {
    pub(in crate::domain_computation) fn durable_is_portable(&self) -> bool {
        self.freshness.durable_view(&self.binding).is_portable()
    }
    pub(in crate::domain_computation) fn durable_view(&self) -> impl serde::Serialize + '_ {
        self.freshness.durable_view(&self.binding)
    }
    pub(in crate::domain_computation) fn capture<Schema, Principal, PrincipalIdentity>(
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        layout: &WorthQueryPrimaryPrincipalBindingLayout,
    ) -> Self {
        Self {
            session_identity,
            binding: Arc::from(principal.binding()),
            layout: layout.clone(),
            freshness: principal.freshness().clone(),
        }
    }

    pub(in crate::domain_computation) const fn session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.session_identity
    }

    pub(in crate::domain_computation) fn retained_for_session(
        &self,
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    ) -> Self {
        Self {
            session_identity,
            binding: Arc::clone(&self.binding),
            layout: self.layout.clone(),
            freshness: self.freshness.clone(),
        }
    }

    pub(in crate::domain_computation) fn remains_current_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<
        bool,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind,
    > {
        self.freshness
            .remains_current_in(execution, runtime, snapshot, &self.layout, &self.binding)
            .map_err(super::super::denial::principal_currentness)
    }
}
