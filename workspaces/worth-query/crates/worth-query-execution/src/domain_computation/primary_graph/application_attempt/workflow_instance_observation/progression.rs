use worth_relational::facade::identity::{EntityId, VersionId};
use worth_relational::facade::runtime::RelationalAdjacencyDirection;

use super::{
    denial, ObservedWorkflowTransition, PublishedWorkflowInstanceRef, SettledWorkflowTransition,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition;
use crate::domain_computation::primary_graph::workflow::instance::{
    select_settled_replay_transition, RetainedWorkflowInstanceProgressProjection,
    WorkflowInstanceProgress, WorkflowInstanceProgressKey, WorkflowInstanceProgressRetentionDenial,
    WorkflowTransitionProgressBasis, WorkflowTransitionProgressObservation,
    WorkflowTransitionReplayProjection, WorkflowTransitionReplayRetention,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;

pub(super) struct WorkflowProgressObservation {
    key: WorkflowInstanceProgressKey,
    revision: Option<VersionId>,
    retained: Option<RetainedWorkflowInstanceProgressProjection>,
}

impl WorkflowProgressObservation {
    pub(super) fn retained_key(&self) -> Option<WorkflowInstanceProgressKey> {
        self.retained.as_ref().map(|_| self.key)
    }
}

pub(super) fn project_replays(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    transitions: &[ObservedWorkflowTransition],
) -> Result<Vec<WorkflowTransitionReplayProjection>, WorthQueryApplicationAttemptDenial> {
    transitions
        .iter()
        .map(|transition| {
            select_settled_replay_transition(compiled, instance, transition.settlement).map(
                |selected| WorkflowTransitionReplayProjection {
                    identity: selected.identity().to_owned(),
                    identity_bytes: *selected.identity_bytes(),
                    node_path: selected.node_path().to_owned(),
                    operation_receipt_identity: transition.settlement.operation_receipt_identity(),
                },
            )
        })
        .collect()
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
    transitions: &[WorkflowTransitionProgressObservation],
    replays: Vec<WorkflowTransitionReplayProjection>,
    reconstruction_transition_visits: usize,
) -> Result<
    (
        WorkflowTransitionProgressBasis,
        WorkflowTransitionReplayRetention,
    ),
    WorthQueryApplicationAttemptDenial,
> {
    match observation.retained {
        Some(retained) => Ok((
            WorkflowTransitionProgressBasis::new(
                observation.key,
                observation.revision,
                compiled.clone(),
                retained.progress,
            ),
            retained.replays,
        )),
        None => {
            let mut settlements = transitions
                .iter()
                .map(|transition| transition.transition().settlement())
                .collect::<Vec<SettledWorkflowTransition>>();
            let mut progress = WorkflowInstanceProgress::reconstruct(compiled, &mut settlements)?;
            for transition in transitions {
                progress.retain_observation(*transition);
            }
            let replays = WorkflowTransitionReplayRetention::from_replays(replays);
            handle
                .with_workflow_instance_progress_mut(observation.key, |retention| {
                    retention.retain(
                        observation.key,
                        observation.revision,
                        progress.clone(),
                        replays.clone(),
                        reconstruction_transition_visits,
                    )
                })
                .map_err(retention_denial)?;
            Ok((
                WorkflowTransitionProgressBasis::new(
                    observation.key,
                    observation.revision,
                    compiled.clone(),
                    progress,
                ),
                replays,
            ))
        }
    }
}

pub(super) fn finish_retained_progress(
    observation: WorkflowProgressObservation,
    compiled: &CompiledWorkflowDefinition,
) -> Result<
    (
        WorkflowTransitionProgressBasis,
        WorkflowTransitionReplayRetention,
    ),
    WorkflowProgressObservation,
> {
    let WorkflowProgressObservation {
        key,
        revision,
        retained,
    } = observation;
    let Some(retained) = retained else {
        return Err(WorkflowProgressObservation {
            key,
            revision,
            retained: None,
        });
    };
    Ok((
        WorkflowTransitionProgressBasis::new(key, revision, compiled.clone(), retained.progress),
        retained.replays,
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
