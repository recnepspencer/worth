use std::collections::HashSet;

use super::{
    adjacency, connection, denial, exact_adjacent, node,
    publication_header::{observe_publication_header, observe_publication_revisions},
    WorkflowDefinitionCompilationPosture,
};
use crate::domain_computation::primary_graph::application_attempt::{
    PublishedWorkflowDefinitionRef, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::{
    definition::compilation::publication_binding::ColdCompiledWorkflowDefinition,
    schema::WorthQueryWorkflowLayout,
};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::identity::{EntityId, KindId};

#[allow(clippy::too_many_arguments)]
pub(super) fn reconstruct_cold_definition(
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
        ColdCompiledWorkflowDefinition,
        Vec<WorthQueryApplicationObservedFact>,
    ),
    WorthQueryApplicationAttemptDenial,
> {
    let observed = observe_publication_header(
        runtime,
        snapshot,
        layout,
        published,
        program_revision,
        expected_spec,
        posture,
        None,
    )?;
    let definition = observed.definition;
    let lineage = observed.lineage;
    let mut facts = observed.facts;
    let revisions =
        observe_publication_revisions(runtime, snapshot, layout, definition, None, &mut facts)?;
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
    // Member fields and endpoints are read during this budgeted cold pass.
    // Their publication is immutable at the native commit boundary, so only
    // the header and complete inventory revisions need start-admission facts.
    let mut member_facts = Vec::new();
    let mut compiled_nodes = Vec::with_capacity(nodes.len());
    for node in &nodes {
        compiled_nodes.push(node::compile_node(
            runtime,
            snapshot,
            layout,
            *node,
            &mut member_facts,
        )?);
        ensure_discarded_member_facts_are_protected(
            &member_facts,
            *node,
            layout.node.entity_kind,
            &[],
        )?;
        member_facts.clear();
    }
    let node_membership: HashSet<_> = nodes.iter().copied().collect();
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
    let mut compiled_connections = Vec::with_capacity(connections.len());
    for connection in &connections {
        compiled_connections.push(connection::compile_connection(
            runtime,
            snapshot,
            layout,
            *connection,
            &node_membership,
            &mut member_facts,
        )?);
        ensure_discarded_member_facts_are_protected(
            &member_facts,
            *connection,
            layout.connection.entity_kind,
            &[
                layout.connection_source_relation,
                layout.connection_target_relation,
            ],
        )?;
        member_facts.clear();
    }
    Ok((
        ColdCompiledWorkflowDefinition {
            lineage,
            definition,
            start_node,
            nodes: compiled_nodes.into_boxed_slice(),
            connections: compiled_connections.into_boxed_slice(),
            revisions,
        },
        facts,
    ))
}

const fn inventory_work_limit(maximum_records: usize) -> usize {
    maximum_records.saturating_mul(2).saturating_add(1)
}

fn ensure_discarded_member_facts_are_protected(
    facts: &[WorthQueryApplicationObservedFact],
    member: EntityId,
    entity_kind: KindId,
    endpoint_relations: &[KindId],
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let protected = facts.iter().all(|fact| match fact {
        WorthQueryApplicationObservedFact::Entity { entity_id, kind }
        | WorthQueryApplicationObservedFact::Field {
            entity_id, kind, ..
        }
        | WorthQueryApplicationObservedFact::AbsentField {
            entity_id, kind, ..
        } => *entity_id == member && *kind == entity_kind,
        WorthQueryApplicationObservedFact::Adjacency {
            relation_kind,
            anchor,
            ..
        } => *anchor == member && endpoint_relations.contains(relation_kind),
        _ => false,
    });
    if protected {
        Ok(())
    } else {
        Err(denial(
            "workflow cold compilation observed a mutable member dependency",
        ))
    }
}
