use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::RelationalAdjacencyDirection;

use super::{
    exact_adjacent, exact_text, exact_u64, observed_optional_text, observed_text,
    WorkflowDefinitionCompilationPosture,
};
use crate::domain_computation::primary_graph::workflow::definition::compilation::publication_binding::WorkflowDefinitionPublicationRevisions;
use crate::domain_computation::primary_graph::application_attempt::{
    PublishedWorkflowDefinitionRef, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

pub(super) struct ObservedWorkflowPublicationHeader {
    pub(super) definition: EntityId,
    pub(super) lineage: EntityId,
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
}

pub(super) fn observe_publication_revisions(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    definition: EntityId,
    expected: Option<&WorkflowDefinitionPublicationRevisions>,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<WorkflowDefinitionPublicationRevisions, WorthQueryApplicationAttemptDenial> {
    let revisions = WorkflowDefinitionPublicationRevisions {
        start: observe_adjacency_revision(
            runtime,
            snapshot,
            layout.definition_start_relation,
            definition,
            facts,
        )?,
        nodes: observe_adjacency_revision(
            runtime,
            snapshot,
            layout.definition_node_relation,
            definition,
            facts,
        )?,
        connections: observe_adjacency_revision(
            runtime,
            snapshot,
            layout.definition_connection_relation,
            definition,
            facts,
        )?,
    };
    if expected.is_some_and(|expected| *expected != revisions) {
        return Err(super::denial(
            "cached workflow publication membership is stale",
        ));
    }
    Ok(revisions)
}

fn observe_adjacency_revision(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: worth_relational::facade::identity::KindId,
    definition: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<worth_relational::facade::identity::VersionId>, WorthQueryApplicationAttemptDenial>
{
    const COMPARISON_WORK_LIMIT: usize = 1;
    let truth = runtime.read_truth();
    let revision = truth
        .project_snapshot(snapshot)
        .ok_or_else(|| super::denial("workflow publication snapshot is unavailable"))?
        .bounded_adjacency_structural_revision(
            definition,
            relation_kind,
            RelationalAdjacencyDirection::Outgoing,
            COMPARISON_WORK_LIMIT,
        )
        .map_err(|_| super::denial("workflow publication membership revision is unavailable"))?
        .revision();
    facts.push(WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
        relation_kind,
        anchor: definition,
        direction: RelationalAdjacencyDirection::Outgoing,
        native_revision: revision,
        comparison_work_limit: COMPARISON_WORK_LIMIT,
        endpoints: Vec::new(),
    });
    Ok(revision)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn observe_publication_header(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    published: &PublishedWorkflowDefinitionRef,
    program_revision: &ApplicationProgramRevision,
    expected_spec: &str,
    posture: WorkflowDefinitionCompilationPosture,
    expected_lineage: Option<EntityId>,
) -> Result<ObservedWorkflowPublicationHeader, WorthQueryApplicationAttemptDenial> {
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
    // Publication provenance is immutable; adoption records a carriage
    // beside it, and the latest carriage names the executing revision.
    let published_under = observed_text(
        runtime,
        snapshot,
        definition,
        layout.definition.entity_kind,
        &layout.definition.program_revision,
        &mut facts,
    )?;
    let carried_to = observed_optional_text(
        runtime,
        snapshot,
        definition,
        layout.definition.entity_kind,
        &layout.definition.carried_revision,
        &mut facts,
    )?;
    if carried_to.unwrap_or(published_under) != program_revision.to_string() {
        return Err(super::denial(
            "workflow definition field does not match installed support",
        ));
    }
    let lineage = exact_adjacent(
        runtime,
        snapshot,
        layout.lineage_definition_relation,
        definition,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        2,
        &mut facts,
    )?;
    if expected_lineage.is_some_and(|expected| lineage != expected) {
        return Err(super::denial(
            "cached workflow publication binding is stale",
        ));
    }
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
        crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        &mut facts,
    )?;
    Ok(ObservedWorkflowPublicationHeader {
        definition,
        lineage,
        facts,
    })
}
