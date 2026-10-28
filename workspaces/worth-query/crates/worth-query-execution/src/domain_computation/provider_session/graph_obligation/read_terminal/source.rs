//! A completed Query source keeps its genuine terminal without the audit review.
use super::*;
use crate::domain_computation::primary_graph::SourceFactsAllocation;
use std::sync::Arc;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(in crate::domain_computation) struct WorthQueryCompletedSourceRead {
    context: super::super::session::WorthQueryQuerySourceContext,
    session: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
    plan: WorthQueryGraphWorkPlanIdentity,
    binding: ApplicationSchemaBindingIdentity,
    obligation: WorthQueryInstalledGraphObligationSetIdentity,
    basis: WorthQueryApplicationBasisIdentity,
    basis_release: WorthQueryApplicationBasisReleaseReceipt,
    dependencies: WorthQueryGraphReadDependencyEvidence,
    release: WorthQueryExecutionCapacityReleaseReceipt,
    _allocation: Arc<SourceFactsAllocation>,
}

impl WorthQueryGraphReadCompletion {
    pub(in crate::domain_computation) fn into_managed_source(
        self,
        context: super::super::session::WorthQueryQuerySourceContext,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation: &Arc<SourceFactsAllocation>,
        work: impl FnOnce(u64) -> Result<(), Cause>,
    ) -> Result<Option<WorthQueryCompletedSourceRead>, Cause> {
        if context.session() != self.session {
            return Ok(None);
        }
        // These are new owned copies. The old ordinary planning/basis backing
        // is dropped, rather than being adopted by a late reservation.
        let bytes = self
            .basis
            .retained_clone_heap_bytes()
            .and_then(|bytes| {
                bytes.checked_add(self.basis_release.identity().retained_clone_heap_bytes()?)
            })
            .ok_or(Cause::ChargedBytesOverflow)?;
        allocation.claim(execution, bytes)?;
        work(bytes)?;
        let basis = self.basis.clone();
        let basis_release = self.basis_release.clone();
        Ok(Some(WorthQueryCompletedSourceRead {
            context,
            session: self.session,
            managed_run: self.managed_run,
            plan: self.plan,
            binding: self.binding,
            obligation: self.obligation,
            basis,
            basis_release,
            dependencies: self.dependencies,
            release: self.release,
            _allocation: Arc::clone(allocation),
        }))
    }
}

impl WorthQueryCompletedSourceRead {
    pub(in crate::domain_computation) fn admits_parent<Stop>(
        &self,
        parent: &super::super::session::WorthQueryManagedGraphWorkSession,
        admit: &mut impl FnMut(u64) -> Result<(), Stop>,
        overflow: impl FnOnce() -> Stop,
    ) -> Result<bool, Stop> {
        self.context.admits_parent(parent, admit, overflow)
    }
    pub(in crate::domain_computation) fn session_identity(
        &self,
    ) -> WorthQueryGraphWorkSessionIdentity {
        self.session
    }
    pub(in crate::domain_computation) fn runtime(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.context.runtime()
    }
    pub(in crate::domain_computation) fn obligation(
        &self,
    ) -> &WorthQueryInstalledGraphObligationSetIdentity {
        &self.obligation
    }
    pub(in crate::domain_computation) fn principal(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.context.principal()
    }
    pub(in crate::domain_computation) fn scope(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.context.scope()
    }
    pub(in crate::domain_computation) fn basis(&self) -> &WorthQueryApplicationBasisIdentity {
        &self.basis
    }
    pub(in crate::domain_computation) fn binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding
    }
}
