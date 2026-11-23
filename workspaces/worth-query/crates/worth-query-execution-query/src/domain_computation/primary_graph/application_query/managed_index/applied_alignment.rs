//! Select the before-basis fast path only when the applied read is aligned.
use super::execution::ProviderExecutionDenial as Denial;
mod footprint;
use crate::domain_computation::primary_graph::{
    application_attempt::AppliedSourceAlignment, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_execution::ExecutionRequest;
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledManagedIndexParent};
use worth_relational::facade::{
    identity::EntityId,
    mvcc::{ManagedIndexParentReadFootprint, RelationalAuthoredCoupledCandidate},
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead},
};

/// The issuing assembly and both pins of an applied decision read.
#[derive(Clone, Copy)]
pub struct AppliedManagedIndexRead<'a> {
    pub assembly: &'a RelationalAuthoredCoupledCandidate<'a>,
    /// The coupled owner that marks later appends against an aligned read.
    pub participants:
        &'a dyn crate::domain_computation::primary_graph::provider::AppliedOutputParticipants,
    pub before: &'a RelationalPinnedRead,
    pub candidate: &'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    /// A successor round's own-output view (PR C §1.5), when it reads.
    pub own: Option<&'a crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
}

impl AppliedManagedIndexRead<'_> {
    /// The retained claim when the selected before basis can supply the whole
    /// Parent read: the coupled owner marks every later append that changes
    /// it, so the committed read keeps exact candidate support.
    pub fn aligned<S: ApplicationSchema>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        association: &WorthQueryInstalledManagedIndexParent,
        scope: EntityId,
        reads: &worth_query_installation::facade::WorthQueryInstalledGraphReadContract,
        parent: &worth_query_declaration::facade::application_query::ApplicationQueryBindingReference,
    ) -> Result<Option<std::sync::Arc<AppliedSourceAlignment>>, Denial> {
        let graph = application
            .runtime
            .primary_graph()
            .ok_or(Denial::ForeignAssociation)?;
        let program = graph
            .layout_ref()
            .managed_source(&association.definition().parts().name)
            .ok_or(Denial::ForeignAssociation)?;
        let footprint = footprint::parent_footprint(graph.layout_ref(), reads, parent)?;
        if !self
            .assembly
            .managed_source_unchanged_at(execution, program, self.candidate, scope, &footprint)
            .map_err(Denial::NativeSource)?
        {
            return Ok(None);
        }
        crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation::work(
            execution, 1,
        )?;
        let alignment = std::sync::Arc::new(AppliedSourceAlignment::new(
            &association.definition().parts().name,
            scope,
            footprint,
            self.candidate.candidate_identity().clone(),
        ));
        self.participants
            .retain_source_alignment(std::sync::Arc::clone(&alignment));
        Ok(Some(alignment))
    }
}
