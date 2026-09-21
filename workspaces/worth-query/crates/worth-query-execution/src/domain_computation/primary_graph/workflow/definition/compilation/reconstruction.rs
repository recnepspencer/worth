use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::identity::{EntityId, KindId};

use super::CompiledWorkflowDefinition;
use crate::domain_computation::primary_graph::application_attempt::{
    observe_adjacency, observe_field_value, PublishedWorkflowDefinitionRef,
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

mod connection;
mod node;

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum WorkflowDefinitionCompilationPosture {
    Current,
    Retained,
}

pub(in crate::domain_computation::primary_graph) fn reconstruct_compiled_definition(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    published: &PublishedWorkflowDefinitionRef,
    program_revision: &ApplicationProgramRevision,
    expected_spec: &str,
    maximum_nodes: usize,
    maximum_connections: usize,
    posture: WorkflowDefinitionCompilationPosture,
) -> Result<
    (
        CompiledWorkflowDefinition,
        Vec<WorthQueryApplicationObservedFact>,
    ),
    WorthQueryApplicationAttemptDenial,
> {
    let definition = published.entity_id();
    let mut facts = vec![WorthQueryApplicationObservedFact::Entity {
        entity_id: definition,
        kind: layout.definition.entity_kind,
    }];
    exact_text(
        runtime,
        snapshot,
        definition,
        layout.definition.entity_kind,
        &layout.definition.content_identity,
        &published.content_identity().to_string(),
        &mut facts,
    )?;
    exact_text(
        runtime,
        snapshot,
        definition,
        layout.definition.entity_kind,
        &layout.definition.program_revision,
        &program_revision.to_string(),
        &mut facts,
    )?;

    let lineage = exact_adjacent(
        runtime,
        snapshot,
        layout.lineage_definition_relation,
        definition,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        2,
        &mut facts,
    )?;
    if matches!(posture, WorkflowDefinitionCompilationPosture::Current) {
        facts.push(
            WorthQueryApplicationObservedFact::WorkflowDefinitionCurrent {
                relation_kind: layout.current_definition_relation,
                lineage,
                expected_definition: definition,
                maximum_work_units: 2,
            },
        );
    }
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: lineage,
        kind: layout.lineage.entity_kind,
    });
    exact_text(
        runtime,
        snapshot,
        lineage,
        layout.lineage.entity_kind,
        &layout.lineage.spec,
        expected_spec,
        &mut facts,
    )?;
    exact_u64(
        runtime,
        snapshot,
        lineage,
        layout.lineage.entity_kind,
        &layout.lineage.protocol_version,
        super::super::super::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        &mut facts,
    )?;
    let start_node = exact_adjacent(
        runtime,
        snapshot,
        layout.definition_start_relation,
        definition,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        2,
        &mut facts,
    )?;
    let nodes = adjacency(
        runtime,
        snapshot,
        layout.definition_node_relation,
        definition,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        inventory_work_limit(maximum_nodes),
        &mut facts,
    )?;
    if nodes.is_empty() || nodes.len() > maximum_nodes || !nodes.contains(&start_node) {
        return Err(denial(
            "published workflow definition node inventory is invalid",
        ));
    }
    let compiled_nodes = nodes
        .iter()
        .map(|node| node::compile_node(runtime, snapshot, layout, *node, &mut facts))
        .collect::<Result<Vec<_>, _>>()?;
    let connections = adjacency(
        runtime,
        snapshot,
        layout.definition_connection_relation,
        definition,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        inventory_work_limit(maximum_connections),
        &mut facts,
    )?;
    if connections.len() > maximum_connections {
        return Err(denial(
            "published workflow definition connection inventory is invalid",
        ));
    }
    let compiled_connections = connections
        .iter()
        .map(|connection| {
            connection::compile_connection(
                runtime,
                snapshot,
                layout,
                *connection,
                &nodes,
                &mut facts,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        CompiledWorkflowDefinition {
            lineage,
            definition,
            content_identity: published.content_identity().clone(),
            program_revision: program_revision.clone(),
            start_node,
            nodes: compiled_nodes.into_boxed_slice(),
            connections: compiled_connections.into_boxed_slice(),
        },
        facts,
    ))
}

const fn inventory_work_limit(maximum_records: usize) -> usize {
    maximum_records.saturating_mul(2).saturating_add(1)
}

fn exact_adjacent(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    limit: usize,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
    let adjacent = adjacency(
        runtime,
        snapshot,
        relation_kind,
        anchor,
        direction,
        limit,
        facts,
    )?;
    match adjacent.as_slice() {
        [entity] => Ok(*entity),
        _ => Err(denial("workflow definition relation is not singular")),
    }
}

fn adjacency(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    limit: usize,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Vec<EntityId>, WorthQueryApplicationAttemptDenial> {
    let relations = observe_adjacency(runtime, snapshot, relation_kind, anchor, direction, limit)
        .ok_or_else(|| denial("workflow definition relation is unavailable"))?;
    let adjacent = relations
        .iter()
        .map(|relation| match direction {
            WorthQueryApplicationAdjacencyDirection::Outgoing => relation.to,
            WorthQueryApplicationAdjacencyDirection::Incoming => relation.from,
        })
        .collect();
    facts.push(WorthQueryApplicationObservedFact::Adjacency {
        relation_kind,
        anchor,
        direction,
        maximum_work_units: limit,
        relations,
    });
    Ok(adjacent)
}

fn exact_text(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    expected: &str,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let value = observed_text(runtime, snapshot, entity, kind, locator, facts)?;
    if value == expected {
        Ok(())
    } else {
        Err(denial(
            "workflow definition field does not match installed support",
        ))
    }
}

fn exact_u64(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    expected: u64,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let value = observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow definition field is unavailable"))?;
    if value != AspectValue::UInt64(expected) {
        return Err(denial("workflow definition protocol is unsupported"));
    }
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value,
    });
    Ok(())
}

fn observed_u64(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<u64, WorthQueryApplicationAttemptDenial> {
    let value = observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow definition field is unavailable"))?;
    let AspectValue::UInt64(value) = value else {
        return Err(denial("workflow definition field has invalid type"));
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: AspectValue::UInt64(value),
    });
    Ok(value)
}

fn observed_optional_u64(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<u64>, WorthQueryApplicationAttemptDenial> {
    let Some(value) = observe_field_value(runtime, snapshot, entity, kind, locator) else {
        facts.push(WorthQueryApplicationObservedFact::AbsentField {
            entity_id: entity,
            kind,
            locator: locator.clone(),
        });
        return Ok(None);
    };
    let AspectValue::UInt64(number) = value else {
        return Err(denial("workflow definition field has invalid type"));
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: AspectValue::UInt64(number),
    });
    Ok(Some(number))
}

fn observed_bool(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<bool, WorthQueryApplicationAttemptDenial> {
    let value = observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow definition field is unavailable"))?;
    let AspectValue::Bool(value) = value else {
        return Err(denial("workflow definition field has invalid type"));
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: AspectValue::Bool(value),
    });
    Ok(value)
}

fn observed_optional_text(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<String>, WorthQueryApplicationAttemptDenial> {
    let Some(value) = observe_field_value(runtime, snapshot, entity, kind, locator) else {
        facts.push(WorthQueryApplicationObservedFact::AbsentField {
            entity_id: entity,
            kind,
            locator: locator.clone(),
        });
        return Ok(None);
    };
    let AspectValue::String(InternedString::Raw(text)) = &value else {
        return Err(denial("workflow definition field has invalid type"));
    };
    let text = text.clone();
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value,
    });
    Ok(Some(text))
}

fn observed_text(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<String, WorthQueryApplicationAttemptDenial> {
    let value = observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow definition field is unavailable"))?;
    let AspectValue::String(InternedString::Raw(text)) = &value else {
        return Err(denial("workflow definition field has invalid type"));
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

fn denial(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionCompilationUnavailable,
        subject,
    )
}
