//! A real Source READ of an unpublished prefix, distinct from positioned completion.
use super::*;
use crate::domain_computation::primary_graph::SourceFactsAllocation;
use std::sync::Arc;
use worth_relational::facade::{
    runtime::{
        ManagedIndexAppliedSourceRead, ManagedIndexSourceProgram,
        RelationalAppliedCandidateReadIdentity,
    },
    transactions::RelationalExecutionDenialCause as Cause,
};

pub struct WorthQueryAppliedSourceRead {
    context: super::super::session::WorthQueryQuerySourceContext,
    session: WorthQueryGraphWorkSessionIdentity,
    _managed_run: WorthQueryGraphWorkManagedRunIdentity,
    _plan: WorthQueryGraphWorkPlanIdentity,
    binding: ApplicationSchemaBindingIdentity,
    obligation: WorthQueryInstalledGraphObligationSetIdentity,
    // C is permission/admission custody only. Source truth is the exact D below.
    _admission_basis: WorthQueryApplicationBasisIdentity,
    _basis_release: WorthQueryApplicationBasisReleaseReceipt,
    _dependencies: WorthQueryGraphReadDependencyEvidence,
    _release: WorthQueryExecutionCapacityReleaseReceipt,
    candidate: RelationalAppliedCandidateReadIdentity,
    program: ManagedIndexSourceProgram,
    publication: worth_relational::facade::runtime::ManagedIndexAppliedSourcePublication,
    _allocation: Arc<SourceFactsAllocation>,
}

impl WorthQueryGraphReadCompletion {
    pub fn into_applied_source(
        self,
        context: super::super::session::WorthQueryQuerySourceContext,
        source: &ManagedIndexAppliedSourceRead<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<Option<WorthQueryAppliedSourceRead>, Cause> {
        if context.session() != self.session || context.scope() != source.scope() {
            return Ok(None);
        }
        let bytes = self
            .basis
            .retained_clone_heap_bytes()
            .and_then(|bytes| {
                bytes.checked_add(self.basis_release.identity().retained_clone_heap_bytes()?)
            })
            .ok_or(Cause::ChargedBytesOverflow)?;
        allocation.claim(execution, bytes)?;
        SourceFactsAllocation::work(execution, bytes)?;
        let basis = self.basis.clone();
        let basis_release = self.basis_release.clone();
        Ok(Some(WorthQueryAppliedSourceRead {
            context,
            session: self.session,
            _managed_run: self.managed_run,
            _plan: self.plan,
            binding: self.binding,
            obligation: self.obligation,
            _admission_basis: basis,
            _basis_release: basis_release,
            _dependencies: self.dependencies,
            _release: self.release,
            candidate: source.candidate_identity().clone(),
            program: source.program().clone(),
            publication: source.publication_support(),
            _allocation: Arc::clone(allocation),
        }))
    }
}

impl WorthQueryAppliedSourceRead {
    pub fn observe_published(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        after: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<
        worth_relational::facade::runtime::ManagedIndexSourceObservation,
        worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial,
    > {
        self.publication
            .observe_published(execution, runtime, after)
    }

    pub fn admits_parent<Stop>(
        &self,
        parent: &super::super::session::WorthQueryManagedGraphWorkSession,
        admit: &mut impl FnMut(u64) -> Result<(), Stop>,
        overflow: impl FnOnce() -> Stop,
    ) -> Result<bool, Stop> {
        self.context.admits_parent(parent, admit, overflow)
    }
    pub fn session_identity(
        &self,
    ) -> WorthQueryGraphWorkSessionIdentity {
        self.session
    }
    pub fn runtime(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.context.runtime()
    }
    pub fn principal(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.context.principal()
    }
    pub fn scope(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.context.scope()
    }
    pub fn binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding
    }
    pub fn obligation(
        &self,
    ) -> &WorthQueryInstalledGraphObligationSetIdentity {
        &self.obligation
    }
    pub fn admits_loan(
        &self,
        source: &ManagedIndexAppliedSourceRead<'_>,
    ) -> bool {
        self.candidate.qualification() == source.candidate_identity().qualification()
            && self.program == *source.program()
            && self.scope() == source.scope()
    }
    pub fn candidate(
        &self,
    ) -> &RelationalAppliedCandidateReadIdentity {
        &self.candidate
    }
}
