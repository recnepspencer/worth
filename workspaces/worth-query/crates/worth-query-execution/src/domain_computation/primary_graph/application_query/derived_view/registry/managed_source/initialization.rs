//! One exclusive seed attempt owns initial construction; lookup never owns it.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::{
        managed_index::SourceInitializationRequest, one_shot::outcome::material::MaterialCompletion,
    },
    source_facts::SourceFactsAllocation,
};
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;

pub(in crate::domain_computation::primary_graph::application_query::derived_view::registry) struct InitializationSession
{
    basis: AdmittedRelationalBranchBasis,
    pub(super) request: SourceInitializationRequest,
    _allocation: Arc<SourceFactsAllocation>,
}
/// Held by the bootstrap caller outside the runtime closure, including refusal.
pub(in crate::domain_computation::primary_graph) struct SourceInitializationGuard {
    owner: Arc<ManagedDerivedViewRegistry>,
    session: Arc<InitializationSession>,
}
impl InitializationSession {
    pub(super) fn applies_to(
        &self,
        source: &ManagedIndexSourcePreparation<'_>,
    ) -> Result<bool, Denial> {
        if !self.request.selected.programs.contains(source.program()) {
            return Ok(false);
        }
        // Equality includes the actual Native admission occurrence, not just a
        // root/commit descriptor. The enclosing guard surrounds one seed commit.
        if self.basis != *source.selected_basis() {
            return Err(Denial::ForeignSource);
        }
        Ok(true)
    }
}
impl ManagedDerivedViewRegistry {
    pub(in crate::domain_computation::primary_graph) fn admits_initialization_binding(
        &self,
        program: &worth_relational::facade::runtime::ManagedIndexSourceProgram,
        binding: std::any::TypeId,
    ) -> bool {
        self.sources
            .service(program)
            .is_some_and(|service| service.binding_type() == binding)
    }
    pub(in crate::domain_computation::primary_graph) fn begin_source_initialization(
        self: &Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        pending: &mut Option<SourceInitializationRequest>,
    ) -> Result<SourceInitializationGuard, Denial> {
        let request = pending.as_ref().ok_or(Denial::ForeignSource)?;
        for program in &request.selected.programs {
            execution.consult().map_err(|cause| {
                Denial::Source(
                    worth_relational::facade::runtime::DerivedRelationReadDenial::Request(
                        cause.into(),
                    ),
                )
            })?;
            if !self
                .sources
                .service(program)
                .is_some_and(|service| service.admits_initialization(&request.proof))
            {
                return Err(Denial::ForeignSource);
            }
        }
        let mut active = self
            .initialization
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if active.strong_count() != 0 {
            return Err(Denial::ForeignSource);
        }
        let allocation = SourceFactsAllocation::open(execution).map_err(|cause| {
            Denial::Source(
                worth_relational::facade::runtime::DerivedRelationReadDenial::Request(cause),
            )
        })?;
        MaterialCompletion::new(execution, &allocation)
            .arc::<InitializationSession>()
            .map_err(|cause| {
                Denial::Source(
                    worth_relational::facade::runtime::DerivedRelationReadDenial::Request(cause),
                )
            })?;
        let session = Arc::new(InitializationSession {
            basis: basis.clone(),
            request: pending
                .take()
                .expect("admission checked this one seed request"),
            _allocation: allocation,
        });
        *active = Arc::downgrade(&session);
        drop(active);
        Ok(SourceInitializationGuard {
            owner: Arc::clone(self),
            session,
        })
    }
}
impl Drop for SourceInitializationGuard {
    fn drop(&mut self) {
        let mut active = self
            .owner
            .initialization
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if Weak::ptr_eq(&active, &Arc::downgrade(&self.session)) {
            *active = Weak::new();
        }
        // session and its retained Native basis leave AFTER the weak lookup lock.
        drop(active);
    }
}
