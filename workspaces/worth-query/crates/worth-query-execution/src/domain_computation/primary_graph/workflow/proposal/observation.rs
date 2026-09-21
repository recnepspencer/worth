use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::primary_graph::application_attempt::{
    observe_adjacency, observe_field_value, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

pub(in crate::domain_computation::primary_graph) fn observe_workflow_operation_input(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    transition_identity: &str,
    expected_operation: &str,
    expected_input_type: &str,
    expected_node_path: &str,
) -> Result<([u8; 32], Vec<WorthQueryApplicationObservedFact>), WorthQueryApplicationAttemptDenial>
{
    let direction = WorthQueryApplicationAdjacencyDirection::Outgoing;
    let relations = observe_adjacency(
        runtime,
        snapshot,
        layout.transition_proposal_relation,
        transition,
        direction,
        2,
    )
    .ok_or_else(|| denial("workflow operation input proposal is unavailable"))?;
    let [relation] = relations.as_slice() else {
        return Err(denial(
            "workflow operation input transition does not own exactly one proposal",
        ));
    };
    let proposal = relation.to;
    let kind = layout.proposal.entity_kind;
    let mut facts = vec![
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
    ];
    let identity = required_text(
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.identity,
        &mut facts,
    )?;
    let operation = required_text(
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.operation,
        &mut facts,
    )?;
    let input_type = required_text(
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.input_type,
        &mut facts,
    )?;
    let input_identity_text = required_text(
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.input_identity,
        &mut facts,
    )?;
    let node_path = required_text(
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.node_path,
        &mut facts,
    )?;
    let source_identity = optional_identity(
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.source_identity,
        &mut facts,
    )?;
    let input_identity = decode_identity(&input_identity_text)
        .ok_or_else(|| denial("workflow operation input identity is malformed"))?;
    let expected = super::derive_workflow_proposal(
        transition_identity,
        &operation,
        &input_type,
        input_identity,
        source_identity,
        &node_path,
    );
    if identity != expected.identity()
        || operation != expected_operation
        || input_type != expected_input_type
        || node_path != expected_node_path
    {
        return Err(denial("workflow operation input proposal changed"));
    }
    Ok((input_identity, facts))
}

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

fn required_text(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<String, WorthQueryApplicationAttemptDenial> {
    let value = observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow operation input field is unavailable"))?;
    let AspectValue::String(InternedString::Raw(text)) = &value else {
        return Err(denial("workflow operation input field has the wrong type"));
    };
    let text = text.clone();
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value,
    });
    Ok(text)
}

fn optional_identity(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<[u8; 32]>, WorthQueryApplicationAttemptDenial> {
    let Some(value) = observe_field_value(runtime, snapshot, entity, kind, locator) else {
        return Ok(None);
    };
    let AspectValue::String(InternedString::Raw(text)) = &value else {
        return Err(denial(
            "workflow operation source identity has the wrong type",
        ));
    };
    let identity = decode_identity(text)
        .ok_or_else(|| denial("workflow operation source identity is malformed"))?;
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value,
    });
    Ok(Some(identity))
}

fn decode_identity(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 {
        return None;
    }
    let mut identity = [0_u8; 32];
    for (index, byte) in identity.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(identity)
}
