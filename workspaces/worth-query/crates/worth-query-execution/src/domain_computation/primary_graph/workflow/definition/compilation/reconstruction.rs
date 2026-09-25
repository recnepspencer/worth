use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::identity::{EntityId, KindId};

use super::plan::CompiledWorkflowDefinition;
use super::publication_binding::{
    separate_compiled_definition, WorkflowDefinitionPublicationBindingDenial,
};
use super::reuse::{
    WorkflowDefinitionCompilationReuseDenial, WorkflowDefinitionPublicationReuseKey,
    WorkflowDefinitionSemanticReuseKey,
};
use crate::domain_computation::primary_graph::application_attempt::{
    observe_adjacency, observe_field_value, PublishedWorkflowDefinitionRef,
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

mod cold;
mod connection;
mod node;
mod publication_header;
use cold::reconstruct_cold_definition;
use publication_header::{observe_publication_header, observe_publication_revisions};

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum WorkflowDefinitionCompilationPosture {
    Current,
    Retained,
}

pub(in crate::domain_computation::primary_graph) fn reconstruct_compiled_definition(
    handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    published: &PublishedWorkflowDefinitionRef,
    program_revision: &ApplicationProgramRevision,
    expected_spec: &str,
    support_identity: &[u8; 32],
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
    let semantic_key = WorkflowDefinitionSemanticReuseKey::new(
        published.content_identity(),
        program_revision,
        expected_spec,
        support_identity,
    );
    let publication_key =
        WorkflowDefinitionPublicationReuseKey::new(published.branch(), published.entity_id());
    let reused = handle.with_workflow_compilation_reuse_mut(
        |reuse: &mut super::reuse::WorkflowDefinitionCompilationReuse| {
            reuse.reuse(publication_key, &semantic_key)
        },
    );
    if let Some(reused) = reused {
        let facts = handle.with_runtime(|runtime| {
            observe_publication_header(
                runtime,
                snapshot,
                layout,
                published,
                program_revision,
                expected_spec,
                posture,
                Some(reused.binding.lineage),
            )
            .and_then(|observed| {
                let mut facts = observed.facts;
                observe_publication_revisions(
                    runtime,
                    snapshot,
                    layout,
                    observed.definition,
                    Some(&reused.binding.revisions),
                    &mut facts,
                )?;
                Ok(facts)
            })
        })?;
        let compiled = reused
            .binding
            .bind(
                reused.semantic,
                published.content_identity().clone(),
                program_revision.clone(),
            )
            .map_err(binding_denial)?;
        return Ok((compiled, facts));
    }
    let (cold, facts) = handle.with_runtime(|runtime| {
        reconstruct_cold_definition(
            runtime,
            snapshot,
            layout,
            published,
            program_revision,
            expected_spec,
            maximum_nodes,
            maximum_connections,
            posture,
        )
    })?;
    let (semantic_candidate, binding) =
        separate_compiled_definition(cold).map_err(binding_denial)?;
    let semantic = handle
        .with_workflow_compilation_reuse_mut(
            |reuse: &mut super::reuse::WorkflowDefinitionCompilationReuse| {
                reuse.retain(
                    publication_key,
                    semantic_key,
                    semantic_candidate,
                    binding.clone(),
                )
            },
        )
        .map_err(reuse_denial)?;
    let compiled = binding
        .bind(
            semantic,
            published.content_identity().clone(),
            program_revision.clone(),
        )
        .map_err(binding_denial)?;
    Ok((compiled, facts))
}

fn binding_denial(
    denial_kind: WorkflowDefinitionPublicationBindingDenial,
) -> WorthQueryApplicationAttemptDenial {
    let message = match denial_kind {
        WorkflowDefinitionPublicationBindingDenial::MissingStartNode => {
            "compiled workflow binding has no start node"
        }
        WorkflowDefinitionPublicationBindingDenial::UnknownConnectionEndpoint => {
            "compiled workflow binding has an unknown connection endpoint"
        }
        WorkflowDefinitionPublicationBindingDenial::BindingCardinalityMismatch => {
            "compiled workflow publication binding cardinality is stale"
        }
    };
    denial(message)
}

fn reuse_denial(
    denial_kind: WorkflowDefinitionCompilationReuseDenial,
) -> WorthQueryApplicationAttemptDenial {
    let message = match denial_kind {
        WorkflowDefinitionCompilationReuseDenial::SemanticCollision => {
            "workflow compilation semantic identity collision was denied"
        }
        WorkflowDefinitionCompilationReuseDenial::ByteBudgetExceeded => {
            "workflow compilation retained-byte budget was exceeded"
        }
    };
    denial(message)
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
