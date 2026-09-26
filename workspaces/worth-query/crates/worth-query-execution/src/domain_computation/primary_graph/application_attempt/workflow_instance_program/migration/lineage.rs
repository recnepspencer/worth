//! What a source's earlier migrations left on it: the one successor it links
//! from once migrated, and the effects it inherited from its own source.

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use super::super::super::{
    observe_adjacency, observe_field_value, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
};
use super::unmapped;
use crate::domain_computation::primary_graph::workflow::{
    instance::WorkflowPerformedEffect, schema::WorthQueryWorkflowLayout,
};

/// The identity of the one successor a migrated source links from.
pub(super) fn successor_identity(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    source: EntityId,
) -> Option<String> {
    let successors = observe_adjacency(
        runtime,
        snapshot,
        layout.instance_migrated_from_relation,
        source,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        2,
    )?;
    let [successor] = successors.as_slice() else {
        return None;
    };
    match observe_field_value(
        runtime,
        snapshot,
        successor.from,
        layout.instance.entity_kind,
        &layout.instance.identity,
    )? {
        AspectValue::String(InternedString::Raw(identity)) => Some(identity),
        _ => None,
    }
}

/// Effects the source carries from its own migration, each at the path of
/// the node that performed it.
pub(super) fn inherited_effects(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    source: EntityId,
    maximum_transitions: usize,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Vec<WorkflowPerformedEffect>, WorthQueryApplicationAttemptDenial> {
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
