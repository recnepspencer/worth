//! The effects an instance has performed: those its own receipted
//! transitions settled, and those it inherited from a migration source.
//! Migration carries them to a successor; cancellation reports them.

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use super::super::workflow_instance_observation::ObservedWorkflowTransition;
use super::super::{
    observe_adjacency, observe_field_value, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::{
    definition::CompiledWorkflowDefinition, instance::WorkflowPerformedEffect,
    schema::WorthQueryWorkflowLayout,
};

/// The instance's own transitions that settled an operation receipt, each at
/// the path of its node in `compiled`.
pub(super) fn own_effects(
    transitions: &[ObservedWorkflowTransition],
    compiled: &CompiledWorkflowDefinition,
    kind: WorthQueryApplicationAttemptDenialKind,
) -> Result<Vec<WorkflowPerformedEffect>, WorthQueryApplicationAttemptDenial> {
    transitions
        .iter()
        .filter(|transition| transition.settlement.operation_receipt_identity().is_some())
        .map(|transition| {
            compiled
                .node(transition.settlement.node())
                .map(|node| WorkflowPerformedEffect {
                    transition: transition.entity,
                    path: node.path().to_owned(),
                })
                .ok_or_else(|| {
                    WorthQueryApplicationAttemptDenial::new(
                        kind,
                        "a performed effect is not in its definition",
                    )
                })
        })
        .collect()
}

/// Effects the instance carries from its own migration, each at the path of
/// the node that performed it. A read that fails is denied as `kind`.
pub(super) fn inherited_effects(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    source: EntityId,
    maximum_transitions: usize,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
    kind: WorthQueryApplicationAttemptDenialKind,
) -> Result<Vec<WorkflowPerformedEffect>, WorthQueryApplicationAttemptDenial> {
    let unmapped = |subject| WorthQueryApplicationAttemptDenial::new(kind, subject);
    let bound = maximum_transitions.saturating_add(1);
    let carried = observe_adjacency(
        runtime,
        snapshot,
        layout.instance_prior_effect_relation,
        source,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        bound,
    )
    .filter(|carried| carried.len() <= maximum_transitions)
    .ok_or_else(|| unmapped("the source's prior effects are unavailable"))?;
    facts.push(WorthQueryApplicationObservedFact::Adjacency {
        relation_kind: layout.instance_prior_effect_relation,
        anchor: source,
        direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
        maximum_work_units: bound,
        relations: carried.clone(),
    });
    let mut inherited = Vec::with_capacity(carried.len());
    for relation in carried {
        let nodes = observe_adjacency(
            runtime,
            snapshot,
            layout.transition_node_relation,
            relation.to,
            WorthQueryApplicationAdjacencyDirection::Outgoing,
            2,
        )
        .ok_or_else(|| unmapped("a prior effect's node is unavailable"))?;
        let [node] = nodes.as_slice() else {
            return Err(unmapped("a prior effect names no single node"));
        };
        let node = node.to;
        facts.push(WorthQueryApplicationObservedFact::Adjacency {
            relation_kind: layout.transition_node_relation,
            anchor: relation.to,
            direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
            maximum_work_units: 2,
            relations: nodes,
        });
        let Some(AspectValue::String(InternedString::Raw(path))) = observe_field_value(
            runtime,
            snapshot,
            node,
            layout.node.entity_kind,
            &layout.node.path,
        ) else {
            return Err(unmapped("a prior effect's node path is unavailable"));
        };
        facts.push(WorthQueryApplicationObservedFact::Field {
            entity_id: node,
            kind: layout.node.entity_kind,
            locator: layout.node.path.clone(),
            value: AspectValue::String(InternedString::Raw(path.clone())),
        });
        inherited.push(WorkflowPerformedEffect {
            transition: relation.to,
            path,
        });
    }
    Ok(inherited)
}
