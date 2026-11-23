//! An applied selection retains real READ and complete physical Source custody.
mod decision;
mod parent;
mod projection;
mod source;
pub mod support;
use super::{
    adapter::InstalledIndexProvider,
    execution::{purpose, ProviderExecutionDenial as Denial},
    source::service::{ManagedSourceRoot, Payload},
    WorthQueryManagedIndexStop as Stop,
};
use crate::domain_computation::{
    primary_graph::{
        access_strategy::coverage::RetainedCoverage,
        application_attempt::SourceReadAuthorization,
        application_query::{
            output_source_observation::SourceQueryReaderAccess, ApplicationQueryNativeAccess,
            WorthQueryObservedScopeSelector,
        },
        source_facts::SourceFactsAllocation,
        WorthQueryPrimaryGraphApplicationRuntime,
    },
    provider_session::{
        WorthQueryAppliedSourceRead, WorthQueryManagedGraphWorkSession,
        WorthQueryManagedIndexSelectionProof,
    },
};
pub(super) use decision::read_decision;
use parent::prepare;
pub use projection::project;
use std::sync::Arc;
pub use support::AppliedManagedSourceSupport;
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::AspectValue;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent;
use worth_relational::facade::{identity::EntityId, runtime::ManagedIndexAppliedSourceRead};

/// Candidate rows cannot impersonate the positioned SelectionRows contract.
pub struct AppliedManagedSelectionLease<'a, 'request, 'authority> {
    proof: WorthQueryManagedIndexSelectionProof<'a, 'request, 'authority>,
    source: Arc<WorthQueryAppliedSourceRead>,
    association: WorthQueryInstalledManagedIndexParent,
    roots: ExecutionArray<EntityId>,
    coverage: Option<Arc<RetainedCoverage>>,
    support: Arc<AppliedManagedSourceSupport>,
    dependencies: Arc<super::super::observed_source::ManagedSourceDependencies>,
    _allocation: Arc<SourceFactsAllocation>,
}
/// A borrow of the actual applied selection. It cannot satisfy positioned rows.
pub struct AppliedSelectionRows<'a> {
    pub association: &'a WorthQueryInstalledManagedIndexParent,
    pub roots: &'a [EntityId],
    pub coverage: Option<&'a Arc<RetainedCoverage>>,
    pub dependencies: &'a Arc<super::super::observed_source::ManagedSourceDependencies>,
    candidate: &'a worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    parent: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    scope: EntityId,
}
impl AppliedSelectionRows<'_> {
    pub fn admits(
        &self,
        parent: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        scope: EntityId,
        candidate: &worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    ) -> bool {
        self.parent == parent && self.scope == scope
            && self.association.binding_identity() == binding
            && self.candidate.qualification() == candidate.qualification()
            && (self.association.definition().parts().membership == worth_query_declaration::facade::application_query::ApplicationManagedIndexMembership::Candidates || self.coverage.is_some())
    }
}
impl AppliedManagedSelectionLease<'_, '_, '_> {
    pub fn rows(
        &self,
        parent: &WorthQueryManagedGraphWorkSession,
        request: &WorthQueryRequestScope,
        candidate: &worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    ) -> Result<AppliedSelectionRows<'_>, Denial> {
        self.proof.consult().map_err(Denial::Session)?;
        if !self
            .proof
            .admits_applied(parent, request, &self.association, &self.source)
            || self.source.candidate().qualification() != candidate.qualification()
            || self.support.candidate().qualification() != candidate.qualification()
        {
            return Err(Denial::ForeignAssociation);
        }
        Ok(AppliedSelectionRows {
            association: &self.association,
            roots: self.roots.elements(),
            coverage: self.coverage.as_ref(),
            dependencies: &self.dependencies,
            candidate: self.source.candidate(),
            parent: parent.identity(),
            scope: self.source.scope(),
        })
    }

    pub fn roots(
        &self,
        parent: &WorthQueryManagedGraphWorkSession,
        request: &WorthQueryRequestScope,
        association: &WorthQueryInstalledManagedIndexParent,
        loan: &ManagedIndexAppliedSourceRead<'_>,
    ) -> Result<&[EntityId], Denial> {
        self.proof.consult().map_err(Denial::Session)?;
        if !self.association.same_association(association)
            || !self
                .proof
                .admits_applied(parent, request, association, &self.source)
            || !self.source.admits_loan(loan)
        {
            return Err(Denial::ForeignAssociation);
        }
        Ok(self.roots.elements())
    }
}
pub fn select<'a, 'request, 'authority, S, I>(
    execution: &ExecutionRequest<'request, 'authority>,
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
    access: &SourceQueryReaderAccess<'_, S>,
    parent: &'a WorthQueryManagedGraphWorkSession,
    request: &'a WorthQueryRequestScope,
    association: &WorthQueryInstalledManagedIndexParent,
    selector: &WorthQueryObservedScopeSelector,
    adapter: &Arc<dyn InstalledIndexProvider<S, I>>,
    before: &worth_relational::facade::runtime::RelationalPinnedRead,
    loan: &ManagedIndexAppliedSourceRead<'_>,
    probe: &AspectValue,
    allocation: &Arc<SourceFactsAllocation>,
    native: &mut ApplicationQueryNativeAccess<'_>,
) -> Result<AppliedManagedSelectionLease<'a, 'request, 'authority>, Denial>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
{
    let admitted = source::acquire(
        execution,
        access,
        parent,
        association,
        selector,
        adapter,
        before,
        loan,
        allocation,
        native,
    )?;
    let root = admitted
        .material
        .downcast_ref::<ManagedSourceRoot<S, I>>()
        .ok_or(Denial::ForeignAssociation)?;
    if !root.installed.same_installation(adapter.installed())
        || !Arc::ptr_eq(adapter, &root.adapter)
    {
        return Err(Denial::ForeignAssociation);
    }
    let purpose = purpose::applied::start(
        execution,
        application,
        parent,
        request,
        association,
        admitted.read.clone(),
        allocation,
    )?;
    let frame =
        crate::domain_computation::primary_graph::access_strategy::coverage::CoverageFrame::applied(
            root.rows.view(&root.bindings, None),
            &root.nodes,
            root,
            probe,
            &admitted.read,
            loan,
            association.access_contract().meaning().match_posture(),
        );
    let (selection, callback) = purpose
        .call(|actual| adapter.candidates(actual, frame))
        .map_err(Denial::Session)?
        .map_err(Denial::Work)?;
    let selected = super::invocation::controlled(execution, |control| {
        selection.receive(frame, adapter.coverage_authority(),
            association.definition().parts().membership != worth_query_declaration::facade::application_query::ApplicationManagedIndexMembership::Candidates,
            control)
    })
    .map_err(Denial::Work)?;
    let proof = purpose.complete(callback).map_err(Denial::Session)?;
    let (slots, coverage) = selected.into_parts();
    let roots = super::invocation::controlled(execution, |control| {
        let mut roots = ExecutionRequestArrayBuilder::allocate(*execution, slots.len())
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        for slot in slots.elements() {
            control.checkpoint(1)?;
            if slot.owner != root.rows.owner() || slot.ordinal != 0 {
                return Err(control.refuse(Stop::ForeignSourceSlot).into());
            }
            let entity = slot
                .entity
                .filter(|entity| root.rows.contains(*entity))
                .ok_or_else(|| control.refuse(Stop::SourceSlotUnavailable))?;
            roots
                .push(entity)
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        }
        roots
            .seal()
            .map_err(|cause| control.refuse(Stop::Storage(cause)).into())
    })
    .map_err(Denial::Work)?;
    let source = Arc::clone(&admitted.read);
    let support = AppliedManagedSourceSupport::retain(
        execution,
        admitted,
        association.clone(),
        coverage.clone(),
        allocation,
    )?;
    let dependencies = super::source_dependencies::capture_applied(execution, &support, allocation)
        .map_err(Denial::SourceCompletion)?;
    let result = AppliedManagedSelectionLease {
        proof,
        roots,
        coverage,
        association: association.clone(),
        source,
        support,
        dependencies,
        _allocation: allocation.clone(),
    };
    result.roots(parent, request, association, loan)?;
    Ok(result)
}
