use worth_relational::facade::identity::VersionId;
use worth_relational::facade::runtime::RelationalAdjacencyDirection;

use super::{
    denial, PublishedWorkflowInstanceRef, SettledWorkflowTransition,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition;
use crate::domain_computation::primary_graph::workflow::instance::{
    WorkflowInstanceProgress, WorkflowInstanceProgressKey, WorkflowInstanceProgressRetentionDenial,
    WorkflowTransitionProgressBasis,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;

pub(super) struct WorkflowProgressObservation {
    key: WorkflowInstanceProgressKey,
    revision: Option<VersionId>,
    retained: Option<WorkflowInstanceProgress>,
}

pub(super) fn observe_progress(
    handle: &WorthQueryPrimaryGraphIntegrationHandle,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: &PublishedWorkflowInstanceRef,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<WorkflowProgressObservation, WorthQueryApplicationAttemptDenial> {
    const COMPARISON_WORK_LIMIT: usize = 1;
    let entity = instance.entity_id();
    let relation_kind = layout.instance_transition_relation;
    let revision = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or_else(|| denial("workflow instance snapshot is unavailable"))?
        .bounded_adjacency_structural_revision(
            entity,
            relation_kind,
            RelationalAdjacencyDirection::Outgoing,
            COMPARISON_WORK_LIMIT,
        )
        .map_err(|_| denial("workflow transition membership revision is unavailable"))?
        .revision();
    facts.push(WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
        relation_kind,
        anchor: entity,
        direction: RelationalAdjacencyDirection::Outgoing,
        native_revision: revision,
        comparison_work_limit: COMPARISON_WORK_LIMIT,
        endpoints: Vec::new(),
    });
    let key = WorkflowInstanceProgressKey::new(
        instance.branch().occurrence_ordinal(),
        entity,
        instance.definition_entity_id(),
    );
    let retained =
        handle.with_workflow_instance_progress_mut(key, |retention| retention.reuse(key, revision));
    Ok(WorkflowProgressObservation {
        key,
        revision,
        retained,
    })
}

pub(super) fn finish_progress(
    handle: &WorthQueryPrimaryGraphIntegrationHandle,
    observation: WorkflowProgressObservation,
    compiled: &CompiledWorkflowDefinition,
    settlements: &mut [SettledWorkflowTransition],
    reconstruction_transition_visits: usize,
) -> Result<WorkflowTransitionProgressBasis, WorthQueryApplicationAttemptDenial> {
    let progress = match observation.retained {
        Some(progress) => progress,
        None => {
            let progress = WorkflowInstanceProgress::reconstruct(compiled, settlements)?;
            handle
                .with_workflow_instance_progress_mut(observation.key, |retention| {
                    retention.retain(
                        observation.key,
                        observation.revision,
                        progress.clone(),
                        reconstruction_transition_visits,
                    )
                })
                .map_err(retention_denial)?;
            progress
        }
    };
    Ok(WorkflowTransitionProgressBasis::new(
        observation.key,
        observation.revision,
        compiled.clone(),
        progress,
    ))
}

fn retention_denial(
    denial: WorkflowInstanceProgressRetentionDenial,
) -> WorthQueryApplicationAttemptDenial {
    let (kind, subject) = match denial {
        WorkflowInstanceProgressRetentionDenial::RevisionCollision => (
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch,
            "retained workflow progress collided at one source revision",
        ),
        WorkflowInstanceProgressRetentionDenial::ContinuityUnavailable => (
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch,
            "retained workflow progress continuity is unavailable",
        ),
        WorkflowInstanceProgressRetentionDenial::ByteBudgetExceeded => (
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
            "workflow progress exceeds its retained byte budget",
        ),
    };
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
