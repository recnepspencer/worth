use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::primary_graph::application_attempt::{
    observe_adjacency, observe_field_value, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

pub(in crate::domain_computation::primary_graph) fn observe_workflow_proposal(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    expected_identity: &str,
) -> Result<Vec<WorthQueryApplicationObservedFact>, WorthQueryApplicationAttemptDenial> {
    let direction = WorthQueryApplicationAdjacencyDirection::Outgoing;
    let relations = observe_adjacency(
        runtime,
        snapshot,
        layout.transition_proposal_relation,
        transition,
        direction,
        2,
    )
    .ok_or_else(|| denial("workflow proposal relation is unavailable"))?;
    let [relation] = relations.as_slice() else {
        return Err(denial(
            "workflow transition does not own exactly one proposal",
        ));
    };
    let proposal = relation.to;
    let kind = layout.proposal.entity_kind;
    let expected = AspectValue::String(InternedString::Raw(expected_identity.to_owned()));
    let value = observe_field_value(runtime, snapshot, proposal, kind, &layout.proposal.identity)
        .ok_or_else(|| denial("workflow proposal identity is unavailable"))?;
    if value != expected {
        return Err(denial("workflow proposal identity changed"));
    }
    Ok(vec![
        WorthQueryApplicationObservedFact::Adjacency {
            relation_kind: layout.transition_proposal_relation,
            anchor: transition,
            direction,
            maximum_work_units: 2,
            relations,
        },
        WorthQueryApplicationObservedFact::Entity {
            entity_id: proposal,
            kind,
        },
        WorthQueryApplicationObservedFact::Field {
            entity_id: proposal,
            kind,
            locator: layout.proposal.identity.clone(),
            value,
        },
    ])
}

fn denial(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
