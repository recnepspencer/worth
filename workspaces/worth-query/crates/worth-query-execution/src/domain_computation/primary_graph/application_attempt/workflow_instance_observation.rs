use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::{EntityId, RelationId};

use super::{
    observe_adjacency, observe_field_value, PublishedWorkflowInstanceRef,
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::instance::{
    SettledWorkflowTransition, WorkflowInstanceState,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

mod settlement;
pub(super) use settlement::{observe_evidence_dependencies, recover_settled_live_membership};

pub(in crate::domain_computation::primary_graph::application_attempt) struct ObservedWorkflowInstance
{
    pub(in crate::domain_computation::primary_graph::application_attempt) live_membership:
        Option<RelationId>,
    pub(in crate::domain_computation::primary_graph::application_attempt) transitions:
        Vec<ObservedWorkflowTransition>,
    pub(in crate::domain_computation::primary_graph::application_attempt) facts:
        Vec<WorthQueryApplicationObservedFact>,
}

pub(in crate::domain_computation::primary_graph::application_attempt) struct ObservedWorkflowTransition
{
    pub(in crate::domain_computation::primary_graph::application_attempt) entity: EntityId,
    pub(in crate::domain_computation::primary_graph::application_attempt) settlement:
        SettledWorkflowTransition,
    pub(in crate::domain_computation::primary_graph::application_attempt) assessment_evidence:
        Option<ObservedWorkflowAssessmentEvidence>,
}

pub(in crate::domain_computation::primary_graph::application_attempt) struct ObservedWorkflowAssessmentEvidence
{
    pub(in crate::domain_computation::primary_graph::application_attempt) entity: EntityId,
    pub(in crate::domain_computation::primary_graph::application_attempt) identity: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) query: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) parameter_type: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) result_type: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) binding: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) passing: bool,
    pub(in crate::domain_computation::primary_graph::application_attempt) source_identity: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) publication_identity:
        String,
    pub(in crate::domain_computation::primary_graph::application_attempt) output_content_identity:
        String,
}

pub(in crate::domain_computation::primary_graph::application_attempt) fn observe_workflow_instance(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: &PublishedWorkflowInstanceRef,
    subject: EntityId,
    lineage: EntityId,
    maximum_transitions: usize,
) -> Result<ObservedWorkflowInstance, WorthQueryApplicationAttemptDenial> {
    let entity = instance.entity_id();
    let kind = layout.instance.entity_kind;
    let mut facts = vec![WorthQueryApplicationObservedFact::Entity {
        entity_id: entity,
        kind,
    }];
    exact(
        runtime,
        snapshot,
        entity,
        kind,
        &layout.instance.protocol_version,
        AspectValue::UInt64(
            crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        ),
        &mut facts,
    )?;
    exact(
        runtime,
        snapshot,
        entity,
        kind,
        &layout.instance.program_revision,
        text(instance.program_revision().to_string()),
        &mut facts,
    )?;
    exact(
        runtime,
        snapshot,
        entity,
        kind,
        &layout.instance.definition_content_identity,
        text(instance.definition_content_identity().to_string()),
        &mut facts,
    )?;
    for (locator, expected) in [
        (
            &layout.instance.subject_partition,
            u64::from(subject.partition_value()),
        ),
        (&layout.instance.subject_slot, subject.local_slot_value()),
        (
            &layout.instance.subject_generation,
            u64::from(subject.generation_value()),
        ),
    ] {
        exact(
            runtime,
            snapshot,
            entity,
            kind,
            locator,
            AspectValue::UInt64(expected),
            &mut facts,
        )?;
    }
    let definitions = adjacency(
        runtime,
        snapshot,
        layout.instance_definition_relation,
        entity,
        2,
        &mut facts,
    )?;
    if definitions.as_slice() != [instance.definition_entity_id()] {
        return Err(denial("workflow instance definition binding changed"));
    }
    let lineages = adjacency(
        runtime,
        snapshot,
        layout.instance_lineage_relation,
        entity,
        2,
        &mut facts,
    )?;
    if lineages.as_slice() != [lineage] {
        return Err(denial("workflow instance lineage binding changed"));
    }
    let live_memberships = observe_adjacency(
        runtime,
        snapshot,
        layout.live_instance_lineage_relation,
        entity,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        2,
    )
    .ok_or_else(|| denial("workflow instance live membership is unavailable"))?;
    let live_membership = match live_memberships.as_slice() {
        [] => None,
        [membership] if membership.to == lineage => Some(membership.relation_id),
        [_] => return Err(denial("workflow instance live lineage changed")),
        _ => return Err(denial("workflow instance has duplicate live membership")),
    };
    facts.push(WorthQueryApplicationObservedFact::Adjacency {
        relation_kind: layout.live_instance_lineage_relation,
        anchor: entity,
        direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
        maximum_work_units: 2,
        relations: live_memberships.clone(),
    });
    exact(
        runtime,
        snapshot,
        entity,
        kind,
        &layout.instance.state,
        AspectValue::UInt64(if live_membership.is_some() {
            WorkflowInstanceState::Ready.persisted_tag()
        } else {
            WorkflowInstanceState::Completed.persisted_tag()
        }),
        &mut facts,
    )?;
    let direction = WorthQueryApplicationAdjacencyDirection::Outgoing;
    let transitions = observe_adjacency(
        runtime,
        snapshot,
        layout.instance_transition_relation,
        entity,
        direction,
        maximum_transitions.saturating_mul(2).saturating_add(1),
    )
    .ok_or_else(|| denial("workflow instance transition inventory is unavailable"))?;
    let transition_count = transitions.len();
    let settled_transitions = transitions
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
            instance: entity,
            maximum_transitions: if live_membership.is_some() {
                maximum_transitions
            } else {
                transition_count
            },
            transitions,
        },
    );
    Ok(ObservedWorkflowInstance {
        live_membership,
        transitions: settled_transitions,
        facts,
    })
}

fn exact_u64(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<u64, WorthQueryApplicationAttemptDenial> {
    let value = observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow transition field is unavailable"))?;
    let AspectValue::UInt64(number) = value else {
        return Err(denial("workflow transition field has the wrong type"));
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: AspectValue::UInt64(number),
    });
    Ok(number)
}

fn exact(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    expected: AspectValue,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let value = observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow instance field is unavailable"))?;
    if value != expected {
        return Err(denial("workflow instance field changed"));
    }
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value,
    });
    Ok(())
}

fn adjacency(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: worth_relational::facade::identity::KindId,
    anchor: EntityId,
    limit: usize,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Vec<EntityId>, WorthQueryApplicationAttemptDenial> {
    adjacency_with_kind(runtime, snapshot, relation_kind, anchor, limit, facts)
}

fn adjacency_with_kind(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: worth_relational::facade::identity::KindId,
    anchor: EntityId,
    limit: usize,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Vec<EntityId>, WorthQueryApplicationAttemptDenial> {
    let direction = WorthQueryApplicationAdjacencyDirection::Outgoing;
    let relations = observe_adjacency(runtime, snapshot, relation_kind, anchor, direction, limit)
        .ok_or_else(|| denial("workflow instance relation is unavailable"))?;
    let adjacent = relations.iter().map(|relation| relation.to).collect();
    facts.push(WorthQueryApplicationObservedFact::Adjacency {
        relation_kind,
        anchor,
        direction,
        maximum_work_units: limit,
        relations,
    });
    Ok(adjacent)
}

fn text(value: String) -> AspectValue {
    AspectValue::String(InternedString::Raw(value))
}

fn denial(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
