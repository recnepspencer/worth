//! Re-admit the complete native inventory before trusting an indexed target.
use super::*;
use crate::domain_computation::primary_graph::workflow::{
    captured_input::{inventory_identity, WorkflowCapturedTarget},
    definition::{CompiledWorkflowDefinition, CompiledWorkflowNodeKind},
};

pub(in crate::domain_computation::primary_graph) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    compiled: &CompiledWorkflowDefinition,
    source: EntityId,
    proposal: EntityId,
    proposal_identity: &str,
    resource: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
    mut preflight: impl FnMut(
        EntityId,
        worth_relational::facade::identity::KindId,
        &worth_foundational::facade::AspectFieldLocator,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>,
) -> Result<Vec<WorkflowCapturedTarget>, WorthQueryApplicationAttemptDenial> {
    let expected_digest = captured::profile(execution, runtime, snapshot, layout, proposal, facts)?
        .ok_or_else(|| denial("captured input source has no native inventory"))?;
    let mut nodes = compiled.captured_input_targets(source).collect::<Vec<_>>();
    nodes.sort_by(|a, b| a.path().cmp(b.path()));
    let count = required_u64(
        execution,
        runtime,
        snapshot,
        proposal,
        layout.proposal.entity_kind,
        &layout.proposal.captured_count,
        facts,
    )?;
    if nodes.is_empty() || usize::try_from(count).ok() != Some(nodes.len()) {
        return Err(denial(
            "captured inventory differs from its declared target count",
        ));
    }
    let limit = nodes
        .len()
        .checked_add(1)
        .and_then(|n| n.checked_mul(2))
        .ok_or_else(|| denial("captured inventory exceeds this host"))?;
    let relations = observe_adjacency(
        runtime,
        execution,
        snapshot,
        layout.captured_input.membership,
        proposal,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        limit,
    )?
    .ok_or_else(|| denial("captured inventory membership is unavailable"))?;
    if relations.len() != nodes.len() {
        return Err(denial("captured inventory membership count changed"));
    }
    for relation in &relations {
        preflight(
            relation.to,
            layout.captured_input.entity_kind,
            &layout.captured_input.identity,
        )?;
    }
    facts.push(WorthQueryApplicationObservedFact::Adjacency {
        relation_kind: layout.captured_input.membership,
        anchor: proposal,
        direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
        maximum_work_units: limit,
        relations,
    });
    let mut targets = Vec::with_capacity(nodes.len());
    for node in nodes {
        let contract = captured::target(
            execution,
            runtime,
            snapshot,
            layout,
            proposal,
            proposal_identity,
            node.path(),
            resource,
            facts,
        )?;
        let CompiledWorkflowNodeKind::Operation {
            binding: Some(binding),
            input_type,
            requires_workflow_authority: true,
            ..
        } = node.kind()
        else {
            return Err(denial(
                "captured inventory target is not a bound guarded operation",
            ));
        };
        if binding != &contract.binding || input_type != &contract.input_type {
            return Err(denial(
                "captured inventory target differs from installed meaning",
            ));
        }
        targets.push(WorkflowCapturedTarget {
            node: node.path().to_owned(),
            contract,
        });
    }
    if inventory_identity(&targets) != Some(expected_digest.0) {
        return Err(denial("captured inventory commitment changed"));
    }
    Ok(targets)
}
