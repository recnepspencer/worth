use worth_relational::facade::identity::EntityId;

use super::{
    denial, observe_adjacency, settlement, ObservedWorkflowTransition,
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

pub(super) struct ObservedWorkflowHistory {
    pub(super) transitions: Vec<ObservedWorkflowTransition>,
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
    pub(super) transition_visits: usize,
}

pub(super) fn observe(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: EntityId,
    live: bool,
    maximum_transitions: usize,
) -> Result<ObservedWorkflowHistory, WorthQueryApplicationAttemptDenial> {
    let direction = WorthQueryApplicationAdjacencyDirection::Outgoing;
    let transitions = observe_adjacency(
        runtime,
        snapshot,
        layout.instance_transition_relation,
        instance,
        direction,
        maximum_transitions.saturating_mul(2).saturating_add(1),
    )
    .ok_or_else(|| denial("workflow instance transition inventory is unavailable"))?;
    let transition_count = transitions.len();
    let mut facts = Vec::new();
    let settled = transitions
        .iter()
        .map(|transition| {
            settlement::observe_settled_transition(
                runtime,
                snapshot,
                layout,
                transition.to,
                &mut facts,
            )
            .and_then(|settlement| {
                settlement::observe_assessment_evidence(
                    runtime,
                    snapshot,
                    layout,
                    transition.to,
                    &mut facts,
                )
                .map(|assessment_evidence| ObservedWorkflowTransition {
                    entity: transition.to,
                    settlement,
                    assessment_evidence,
                })
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    facts.push(
        WorthQueryApplicationObservedFact::WorkflowTransitionCapacity {
            relation_kind: layout.instance_transition_relation,
            instance,
            maximum_transitions: if live {
                maximum_transitions
            } else {
                transition_count
            },
            transitions,
        },
    );
    Ok(ObservedWorkflowHistory {
        transitions: settled,
        facts,
        transition_visits: transition_count,
    })
}
