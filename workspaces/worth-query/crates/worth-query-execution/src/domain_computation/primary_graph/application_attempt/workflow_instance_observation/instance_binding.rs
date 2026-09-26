use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::EntityId;

use super::{
    denial, observe_adjacency, observe_field_value, CompiledWorkflowDefinition,
    PublishedWorkflowInstanceRef, WorkflowInstanceState, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact, WorthQueryWorkflowLayout,
};

/// An ended instance is named before any field it left behind can mismatch.
pub(super) fn deny_ended(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    entity: EntityId,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let kind = layout.instance.entity_kind;
    let state = observe_field_value(runtime, snapshot, entity, kind, &layout.instance.state);
    for (ended, denial_kind, subject) in [
        (
            WorkflowInstanceState::Cancelled,
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled,
            "workflow instance was cancelled by program adoption",
        ),
        (
            WorkflowInstanceState::Migrated,
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrated,
            "workflow instance was migrated to a successor",
        ),
    ] {
        if state == Some(AspectValue::UInt64(ended.persisted_tag())) {
            return Err(WorthQueryApplicationAttemptDenial::new(
                denial_kind,
                subject,
            ));
        }
    }
    Ok(())
}

/// Reads the definition as this instance runs it: from the node its migration
/// resumed at, which its reference names, or from the definition's start.
pub(super) fn resume_definition(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: &PublishedWorkflowInstanceRef,
    compiled: &mut CompiledWorkflowDefinition,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let entity_id = instance.entity_id();
    let kind = layout.instance.entity_kind;
    let locator = &layout.instance.resume_node_path;
    match observe_field_value(runtime, snapshot, entity_id, kind, locator) {
        None => {
            facts.push(WorthQueryApplicationObservedFact::AbsentField {
                entity_id,
                kind,
                locator: locator.clone(),
            });
            Ok(())
        }
        Some(AspectValue::String(InternedString::Raw(path))) => {
            if path != instance.start_node_path() {
                return Err(denial("workflow instance resume node changed"));
            }
            *compiled = compiled
                .resumed_at(&path)
                .ok_or_else(|| denial("workflow instance resume node is not in its definition"))?;
            facts.push(WorthQueryApplicationObservedFact::Field {
                entity_id,
                kind,
                locator: locator.clone(),
                value: text(path),
            });
            Ok(())
        }
        Some(_) => Err(denial("workflow instance resume node has the wrong type")),
    }
}

pub(super) fn exact_u64(
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

pub(super) fn exact(
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

pub(super) fn adjacency(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: worth_relational::facade::identity::KindId,
    anchor: EntityId,
    limit: usize,
    unavailable: &'static str,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Vec<EntityId>, WorthQueryApplicationAttemptDenial> {
    adjacency_with_kind(
        runtime,
        snapshot,
        relation_kind,
        anchor,
        limit,
        unavailable,
        facts,
    )
}

pub(super) fn adjacency_with_kind(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: worth_relational::facade::identity::KindId,
    anchor: EntityId,
    limit: usize,
    unavailable: &'static str,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Vec<EntityId>, WorthQueryApplicationAttemptDenial> {
    let direction = WorthQueryApplicationAdjacencyDirection::Outgoing;
    let relations = observe_adjacency(runtime, snapshot, relation_kind, anchor, direction, limit)
        .ok_or_else(|| denial(unavailable))?;
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

pub(super) fn text(value: String) -> AspectValue {
    AspectValue::String(InternedString::Raw(value))
}
