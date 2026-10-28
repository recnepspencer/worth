//! Exact session correspondence for each real application read subject.
use super::*;
use std::sync::Arc;
use worth_query_installation::facade::{
    WorthQueryInstalledResourceAccessContract, WorthQueryResourceAccessPurpose,
};

enum DisclosureSubject {
    Query(WorthQueryInstalledApplicationQueryIdentity),
    Resource(Arc<WorthQueryInstalledResourceAccessContract>),
}

pub(in crate::domain_computation) struct WorthQueryApplicationGovernanceBinding {
    runtime: WorthQueryRuntimeAuthorityIdentity,
    subject: DisclosureSubject,
    parameters: CanonicalDigestId,
    principal: EntityId,
    scope: EntityId,
    session: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
    branch: BranchId,
    basis: crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity,
    provider: String,
}

impl WorthQueryApplicationGovernanceBinding {
    pub(super) fn is_resource(&self) -> bool {
        matches!(&self.subject, DisclosureSubject::Resource(_))
    }
    pub(in crate::domain_computation) fn from_session(
        graph_work: &WorthQueryManagedGraphWorkSession,
        query: WorthQueryInstalledApplicationQueryIdentity,
        parameters: CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> Self {
        Self::bound(
            graph_work,
            DisclosureSubject::Query(query),
            parameters,
            principal,
            scope,
            graph_work
                .query_basis()
                .expect("application Query disclosure requires a Query session")
                .clone(),
        )
    }

    pub(in crate::domain_computation) fn from_resource_session(
        graph_work: &WorthQueryManagedGraphWorkSession,
        contract: &Arc<WorthQueryInstalledResourceAccessContract>,
        parameters: CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> Option<Self> {
        let (actual, basis) = graph_work.resource_read_binding()?;
        if !Arc::ptr_eq(actual, contract) {
            return None;
        }
        Some(Self::bound(
            graph_work,
            DisclosureSubject::Resource(Arc::clone(actual)),
            parameters,
            principal,
            scope,
            basis.clone(),
        ))
    }

    fn bound(
        graph_work: &WorthQueryManagedGraphWorkSession,
        subject: DisclosureSubject,
        parameters: CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
        basis: crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity,
    ) -> Self {
        Self {
            runtime: graph_work.runtime_authority(),
            subject,
            parameters,
            principal,
            scope,
            session: graph_work.identity(),
            managed_run: graph_work.managed_run_identity(),
            branch: graph_work.branch().relational().clone(),
            basis,
            provider: graph_work.provider().to_string(),
        }
    }

    pub(super) fn matches(
        &self,
        graph_work: &WorthQueryManagedGraphWorkSession,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        query: &WorthQueryInstalledApplicationQueryIdentity,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> bool {
        self.readmission_matches(runtime, query, parameters, principal, scope)
            && self.matches_session(graph_work)
            && graph_work.query_basis() == Some(&self.basis)
    }

    pub(super) fn readmission_matches(
        &self,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        query: &WorthQueryInstalledApplicationQueryIdentity,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> bool {
        self.runtime == runtime
            && matches!(&self.subject, DisclosureSubject::Query(actual) if actual == query)
            && &self.parameters == parameters
            && self.principal == principal
            && self.scope == scope
    }

    pub(super) fn matches_resource(
        &self,
        graph_work: &WorthQueryManagedGraphWorkSession,
        contract: &Arc<WorthQueryInstalledResourceAccessContract>,
        purpose: WorthQueryResourceAccessPurpose,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> bool {
        let Some((actual, basis)) = graph_work.resource_read_binding() else {
            return false;
        };
        matches!(&self.subject, DisclosureSubject::Resource(retained) if Arc::ptr_eq(retained, contract))
            && Arc::ptr_eq(actual, contract)
            && contract.purpose() == purpose
            && basis == &self.basis
            && self.matches_session(graph_work)
            && &self.parameters == parameters
            && self.principal == principal
            && self.scope == scope
    }

    fn matches_session(&self, graph_work: &WorthQueryManagedGraphWorkSession) -> bool {
        self.runtime == graph_work.runtime_authority()
            && self.session == graph_work.identity()
            && self.managed_run == graph_work.managed_run_identity()
            && self.branch == *graph_work.branch().relational()
            && self.provider == graph_work.provider()
    }
}
