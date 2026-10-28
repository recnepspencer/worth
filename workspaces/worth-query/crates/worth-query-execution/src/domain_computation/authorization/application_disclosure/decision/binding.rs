//! Exact session correspondence for each real application read subject.
use super::*;
use std::sync::Arc;
use worth_query_installation::facade::{
    WorthQueryInstalledResourceAccessContract, WorthQueryResourceAccessPurpose,
};

enum DisclosureSubject {
    Query(WorthQueryInstalledApplicationQueryIdentity),
    Resource(Arc<WorthQueryInstalledResourceAccessContract>),
    Inspection(Arc<crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionTarget>),
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

    pub(in crate::domain_computation) fn inspection_backing_bytes(
        graph_work: &WorthQueryManagedGraphWorkSession,
    ) -> Option<u64> {
        let (_, basis) = graph_work.inspection_binding()?;
        let names = graph_work
            .branch()
            .relational()
            .0
            .len()
            .checked_add(graph_work.provider().len())?;
        basis
            .retained_clone_heap_bytes()?
            .checked_add(u64::try_from(names).ok()?)
    }

    pub(in crate::domain_computation) fn from_inspection_session(
        graph_work: &WorthQueryManagedGraphWorkSession,
        target: &Arc<crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionTarget>,
        parameters: CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> Option<Self> {
        let (actual, basis) = graph_work.inspection_binding()?;
        if !Arc::ptr_eq(actual, target)
            || graph_work.principal() != principal
            || graph_work.entity_access_context() != Some(scope)
        {
            return None;
        }
        Some(Self::bound(
            graph_work,
            DisclosureSubject::Inspection(Arc::clone(actual)),
            parameters,
            principal,
            scope,
            basis.clone(),
        ))
    }

    pub(super) fn is_inspection(&self) -> bool {
        matches!(&self.subject, DisclosureSubject::Inspection(_))
    }

    pub(super) fn matches_inspection(
        &self,
        graph_work: &WorthQueryManagedGraphWorkSession,
        target: &Arc<crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionTarget>,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> bool {
        let Some((actual, basis)) = graph_work.inspection_binding() else {
            return false;
        };
        matches!(&self.subject, DisclosureSubject::Inspection(retained) if Arc::ptr_eq(retained, target))
            && Arc::ptr_eq(actual, target)
            && basis == &self.basis
            && self.matches_session(graph_work)
            && &self.parameters == parameters
            && self.principal == principal
            && self.scope == scope
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
