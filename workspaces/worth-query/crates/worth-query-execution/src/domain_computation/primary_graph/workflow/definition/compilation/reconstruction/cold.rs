use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

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
