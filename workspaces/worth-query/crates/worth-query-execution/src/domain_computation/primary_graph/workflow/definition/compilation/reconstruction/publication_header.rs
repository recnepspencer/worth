use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::{
    AdjacencyStructuralRevisionDenial, RelationalAdjacencyDirection,
};

use super::WorkflowDefinitionCompilationPosture;
mod recorded;
use crate::domain_computation::primary_graph::workflow::definition::compilation::publication_binding::WorkflowDefinitionPublicationRevisions;
use crate::domain_computation::primary_graph::application_attempt::{
    PublishedWorkflowDefinitionRef,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

pub(super) struct ObservedWorkflowPublicationHeader {
    pub(super) definition: EntityId,
    pub(super) lineage: EntityId,
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
}

pub(super) fn observe_publication_revisions(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    definition: EntityId,
    expected: Option<&WorkflowDefinitionPublicationRevisions>,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<WorkflowDefinitionPublicationRevisions, WorthQueryApplicationAttemptDenial> {
    let revisions = WorkflowDefinitionPublicationRevisions {
        start: observe_adjacency_revision(
            execution,
            runtime,
            snapshot,
            layout.definition_start_relation,
            definition,
            facts,
        )?,
        nodes: observe_adjacency_revision(
            execution,
            runtime,
            snapshot,
            layout.definition_node_relation,
            definition,
            facts,
        )?,
        connections: observe_adjacency_revision(
            execution,
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
            execution,
            definition,
            relation_kind,
            RelationalAdjacencyDirection::Outgoing,
            COMPARISON_WORK_LIMIT,
        )
        .map_err(|unavailable| match unavailable {
            AdjacencyStructuralRevisionDenial::Execution(cause) => {
                WorthQueryApplicationAttemptDenial::relational_read_stopped(
                    cause,
                    "workflow publication revision",
                )
            }
            AdjacencyStructuralRevisionDenial::WorkBudgetExceeded => super::denial(
                "workflow publication membership revision exceeds its comparison work",
            ),
            AdjacencyStructuralRevisionDenial::AnchorUnavailable => {
                super::denial("workflow definition is not visible for its membership revision")
            }
            AdjacencyStructuralRevisionDenial::BasisUnavailable => {
                super::denial("workflow publication membership revision is unavailable")
            }
        })?
        .revision();
    facts.push(WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
        relation_kind,
        anchor: definition,
        direction: RelationalAdjacencyDirection::Outgoing,
        native_revision: revision,
        comparison_work_limit: COMPARISON_WORK_LIMIT,
        endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::empty(),
    });
    Ok(revision)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn observe_publication_header(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
    let recorded = recorded::observe(execution, runtime, snapshot, layout, definition)?;
    if &recorded.content != published.content_identity() {
        return Err(super::denial(
            "workflow definition content identity changed",
        ));
    }
    // Recorded provenance remains DATA. Only this executable wrapper compares
    // it against the caller's genuinely installed program and vocabulary.
    if recorded
        .carried_to
        .unwrap_or(recorded.published_under)
        .bytes()
        != program_revision.as_bytes()
    {
        return Err(super::denial(
            "workflow definition field does not match installed support",
        ));
    }
    let lineage = recorded.lineage;
    if expected_lineage.is_some_and(|expected| lineage != expected) {
        return Err(super::denial(
            "cached workflow publication binding is stale",
        ));
    }
    if recorded.spec != expected_spec {
        return Err(super::denial("workflow definition vocabulary changed"));
    }
    let mut facts = recorded.facts;
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
    Ok(ObservedWorkflowPublicationHeader {
        definition,
        lineage,
        facts,
    })
}
