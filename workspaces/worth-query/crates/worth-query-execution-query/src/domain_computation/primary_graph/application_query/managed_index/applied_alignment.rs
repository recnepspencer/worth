//! Select the before-basis fast path only when the applied read is aligned.
use super::execution::ProviderExecutionDenial as Denial;
mod footprint;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
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
    pub before: &'a RelationalPinnedRead,
    pub candidate: &'a RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
}

impl AppliedManagedIndexRead<'_> {
    /// Whether the selected before basis can supply the whole Parent read.
    pub fn is_aligned<S: ApplicationSchema>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        association: &WorthQueryInstalledManagedIndexParent,
        scope: EntityId,
        reads: &worth_query_installation::facade::WorthQueryInstalledGraphReadContract,
        parent: &worth_query_declaration::facade::application_query::ApplicationQueryBindingReference,
    ) -> Result<bool, Denial> {
        let graph = application
            .runtime
            .primary_graph()
            .ok_or(Denial::ForeignAssociation)?;
        let program = graph
            .layout_ref()
            .managed_source(&association.definition().parts().name)
            .ok_or(Denial::ForeignAssociation)?;
        let footprint = footprint::parent_footprint(graph.layout_ref(), reads, parent)?;
        self.assembly
            .managed_source_unchanged_at(execution, program, self.candidate, scope, &footprint)
            .map_err(Denial::NativeSource)
    }
}
