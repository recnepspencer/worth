use std::sync::Arc;

use crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
use crate::domain_computation::primary_graph::WorthQueryPrimaryPrincipalBindingLayout;
use crate::domain_computation::primary_graph::WorthQueryPrincipalFreshnessEvidence;

#[derive(Clone)]
pub struct WorthQueryPrincipalCurrentnessDependency {
    session_identity:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    binding: Arc<str>,
    layout: WorthQueryPrimaryPrincipalBindingLayout,
    freshness: WorthQueryPrincipalFreshnessEvidence,
    external_identity:
        Arc<worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity>,
}

impl WorthQueryPrincipalCurrentnessDependency {
    pub fn durable_is_portable(&self) -> bool {
        self.freshness.durable_view(&self.binding).is_portable()
    }
    pub fn durable_view(&self) -> impl serde::Serialize + '_ {
        self.freshness.durable_view(&self.binding)
    }
    pub fn capture<Schema, Principal, PrincipalIdentity>(
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        layout: &WorthQueryPrimaryPrincipalBindingLayout,
    ) -> Self {
        Self {
            session_identity,
            binding: Arc::from(principal.binding()),
            layout: layout.clone(),
            freshness: principal.freshness().clone(),
            external_identity: Arc::new(principal.external_identity().clone()),
        }
    }

    pub const fn session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.session_identity
    }

    /// Typed identity resolved by native admission, not a decoded durable snapshot.
    pub fn external_identity(
        &self,
    ) -> &Arc<worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity>
    {
        &self.external_identity
    }

    pub fn retained_for_session(
        &self,
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    ) -> Self {
        Self {
            session_identity,
            binding: Arc::clone(&self.binding),
            layout: self.layout.clone(),
            freshness: self.freshness.clone(),
            external_identity: Arc::clone(&self.external_identity),
        }
    }

    pub(super) fn source_query_facts(
        &self,
        admit: impl FnOnce(usize) -> Result<(), worth_relational::facade::mvcc::CompanionPreflightStop>,
    ) -> Result<
        Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
        worth_relational::facade::mvcc::CompanionPreflightStop,
    > {
        self.freshness.source_query_facts(&self.layout, admit)
    }

    pub fn remains_current_in(
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

pub mod court_doors_for_test;
