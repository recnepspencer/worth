use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::{EntityId, RelationId};

use super::{
    observe_adjacency, observe_field_value, PublishedWorkflowInstanceRef,
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition;
use crate::domain_computation::primary_graph::workflow::instance::{
    SettledWorkflowTransition, WorkflowInstanceState, WorkflowTransitionLocator,
    WorkflowTransitionProgressBasis, WorkflowTransitionProgressObservation,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

mod history;
mod instance_binding;
mod progression;
mod settlement;
use instance_binding::{adjacency, adjacency_with_kind, exact, exact_u64, text};
pub(super) use settlement::{
    observe_evidence_dependencies, observe_retained_assessment_evidence,
    observe_retained_transition, observe_retained_workflow_proposal_identity,
    recover_settled_live_membership,
};

pub(in crate::domain_computation::primary_graph::application_attempt) struct ObservedWorkflowInstance
{
    pub(in crate::domain_computation::primary_graph::application_attempt) live_membership:
        Option<RelationId>,
    pub(in crate::domain_computation::primary_graph::application_attempt) transitions:
        Vec<ObservedWorkflowTransition>,
    pub(in crate::domain_computation::primary_graph::application_attempt) facts:
        Vec<WorthQueryApplicationObservedFact>,
    pub(in crate::domain_computation::primary_graph::application_attempt) progress_basis:
        WorkflowTransitionProgressBasis,
    pub(in crate::domain_computation::primary_graph::application_attempt) replays:
        crate::domain_computation::primary_graph::workflow::instance::WorkflowTransitionReplayRetention,
    history_materialized: bool,
}

impl ObservedWorkflowInstance {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn ensure_history(
        &mut self,
        handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        layout: &WorthQueryWorkflowLayout,
        instance: EntityId,
        maximum_transitions: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if self.history_materialized {
            return Ok(());
        }
        let mut history = history::observe(
            runtime,
            snapshot,
            layout,
            instance,
            self.live_membership.is_some(),
            maximum_transitions,
        )?;
        let transition_visits = history.transition_visits;
        self.transitions = history.transitions;
        let (key, _) = self.progress_basis.replay_retention();
        handle.with_workflow_instance_progress_mut(key, |retention| {
            retention.observe_warm_history(transition_visits)
        });
        self.facts.append(&mut history.facts);
        self.history_materialized = true;
        Ok(())
    }
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
    pub(in crate::domain_computation::primary_graph::application_attempt) coverage_identity: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) source_identity: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) publication_identity:
        String,
    pub(in crate::domain_computation::primary_graph::application_attempt) output_content_identity:
        String,
}

pub(in crate::domain_computation::primary_graph::application_attempt) fn latest_transition_for_node(
    transitions: &[ObservedWorkflowTransition],
    node: EntityId,
) -> Option<&ObservedWorkflowTransition> {
    transitions
        .iter()
        .filter(|transition| transition.settlement.node() == node)
        .max_by_key(|transition| transition.settlement.occurrence())
}

pub(in crate::domain_computation::primary_graph::application_attempt) fn latest_assessment_evidence(
    transitions: &[ObservedWorkflowTransition],
    node: EntityId,
) -> Option<&ObservedWorkflowAssessmentEvidence> {
    transitions
        .iter()
        .filter(|transition| transition.settlement.node() == node)
        .filter_map(|transition| {
            transition
                .assessment_evidence
                .as_ref()
                .map(|evidence| (transition.settlement.occurrence(), evidence))
        })
        .max_by_key(|(occurrence, _)| *occurrence)
        .map(|(_, evidence)| evidence)
}

pub(in crate::domain_computation::primary_graph::application_attempt) fn observe_workflow_instance(
    handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: &PublishedWorkflowInstanceRef,
    subject: EntityId,
    lineage: EntityId,
    compiled: &CompiledWorkflowDefinition,
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
        "workflow instance definition relation is unavailable",
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
        "workflow instance lineage relation is unavailable",
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
    let progress_observation =
        progression::observe_progress(handle, runtime, snapshot, layout, instance, &mut facts)?;
    let retained_history_key = live_membership
        .is_none()
        .then(|| progress_observation.retained_key())
        .flatten();
    let progress_observation = if live_membership.is_some() {
        match progression::finish_retained_progress(progress_observation, compiled) {
            Ok((progress_basis, replays)) => {
                let next_occurrence = usize::try_from(progress_basis.progress().next_occurrence())
                    .unwrap_or(usize::MAX);
                if next_occurrence >= maximum_transitions {
                    return Err(WorthQueryApplicationAttemptDenial::new(
                        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
                        "workflow instance transition capacity is exhausted",
                    ));
                }
                let (key, _) = progress_basis.replay_retention();
                handle.with_workflow_instance_progress_mut(key, |retention| {
                    retention.observe_warm_core()
                });
                return Ok(ObservedWorkflowInstance {
                    live_membership,
                    transitions: Vec::new(),
                    facts,
                    progress_basis,
                    replays,
                    history_materialized: false,
                });
            }
            Err(observation) => observation,
        }
    } else {
        progress_observation
    };
    let mut history = history::observe(
        runtime,
        snapshot,
        layout,
        entity,
        live_membership.is_some(),
        maximum_transitions,
    )?;
    let transition_visits = history.transition_visits;
    let settled_transitions = history.transitions;
    if let Some(key) = retained_history_key {
        handle.with_workflow_instance_progress_mut(key, |retention| {
            retention.observe_warm_history(transition_visits)
        });
    }
    let mut progress_observations = settled_transitions
        .iter()
        .map(|transition| {
            WorkflowTransitionProgressObservation::new(
                WorkflowTransitionLocator::new(transition.entity, transition.settlement),
                transition
                    .assessment_evidence
                    .as_ref()
                    .map(|evidence| evidence.entity),
            )
        })
        .collect::<Vec<_>>();
    if live_membership.is_none() {
        let settled_index = progress_observations
            .iter()
            .enumerate()
            .max_by_key(|(_, observation)| observation.transition().settlement().occurrence())
            .map(|(index, _)| index);
        if let Some(index) = settled_index {
            progress_observations.swap_remove(index);
        }
    }
    let reconstruction_transition_visits = progress_observations.len();
    let replay_projections = progression::project_replays(compiled, entity, &settled_transitions)?;
    let (progress_basis, replays) = progression::finish_progress(
        handle,
        progress_observation,
        compiled,
        &progress_observations,
        replay_projections,
        reconstruction_transition_visits,
    )?;
    facts.append(&mut history.facts);
    Ok(ObservedWorkflowInstance {
        live_membership,
        transitions: settled_transitions,
        facts,
        progress_basis,
        replays,
        history_materialized: true,
    })
}

fn denial(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
