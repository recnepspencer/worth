//! Cold inventory reconstruction reads actual graph membership before retention.
use super::*;
use {
    crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::WorkflowCapturedInputSupport,
    crate::domain_computation::primary_graph::application_attempt::workflow::definition::CompiledWorkflowDefinition,
};
use crate::domain_computation::primary_graph::vocabulary::application_attempt::fact::WorthQueryApplicationAdjacencyDirection;
pub(in crate::domain_computation::primary_graph) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    compiled: &CompiledWorkflowDefinition,
    source: EntityId,
    transition: EntityId,
    resource: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
    mut preflight: impl FnMut(
        EntityId,
        worth_relational::facade::identity::KindId,
        &worth_foundational::facade::AspectFieldLocator,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>,
) -> Result<std::sync::Arc<WorkflowCapturedInputSupport>, WorthQueryApplicationAttemptDenial> {
    let proposal = crate::domain_computation::primary_graph::application_attempt::workflow::proposal::observation::captured::one_reference(
        execution,
        runtime,
        snapshot,
        layout.transition_proposal_relation,
        transition,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        facts,
    )?;
    preflight(
        proposal,
        layout.proposal.entity_kind,
        &layout.proposal.identity,
    )?;
    let identity = required_text(
        execution,
        runtime,
        snapshot,
        proposal,
        layout.proposal.entity_kind,
        &layout.proposal.identity,
        facts,
    )?;
    let digest = crate::domain_computation::primary_graph::application_attempt::workflow::proposal::observation::captured::profile(execution, runtime, snapshot, layout, proposal, facts)?
        .ok_or_else(|| denial("captured history has no committed inventory"))?;
    let targets = crate::domain_computation::primary_graph::application_attempt::workflow::proposal::observation::captured_inventory::observe(
        execution, runtime, snapshot, layout, compiled, source, proposal, &identity, resource,
        facts, preflight,
    )?;
    WorkflowCapturedInputSupport::validate(identity, digest.0, targets).map(std::sync::Arc::new)
}
