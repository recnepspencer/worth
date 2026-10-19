pub(in crate::domain_computation::primary_graph) mod history_projection;
mod read_dependencies;
pub(in crate::domain_computation::primary_graph) mod read_fields;
mod recorded_actor;
pub(in crate::domain_computation::primary_graph::application_attempt) use recorded_actor::{
    original_parent, original_scope, original_transition, recorded_approval,
};
mod actors;
pub(in crate::domain_computation::primary_graph) mod audit;
pub(in crate::domain_computation::primary_graph) mod governed_record;
mod governed_start_path;
mod transition_roles;
pub(in crate::domain_computation::primary_graph::application_attempt) use actors::observe as observe_instance_roles;
pub(in crate::domain_computation::primary_graph::application_attempt) use transition_roles::observe as observe_transition_roles;
pub(in crate::domain_computation::primary_graph) mod read_only;
mod retained_history;
pub(in crate::domain_computation::primary_graph) use read_only::ReadOnlyWorkflowHistory;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::{EntityId, RelationId};

use super::{
    observe_adjacency, observe_field_value, PublishedWorkflowInstanceRef,
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition;
use crate::domain_computation::primary_graph::workflow::instance::{
    SettledWorkflowTransition, WorkflowInstanceState, WorkflowTransitionProgressBasis,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

mod completed_source;
mod evidence_budget;
mod history;
pub(super) use history::observe_ended_history;
pub(in crate::domain_computation::primary_graph::application_attempt) mod instance_binding;
mod progression;
pub(in crate::domain_computation::primary_graph::application_attempt) mod settlement;
mod step_budget;
mod subject;
use instance_binding::{adjacency, adjacency_with_kind, exact, exact_u64, text};
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use settlement::decode_field_revision_fact;
pub(super) use settlement::{
    observe_evidence_dependencies, observe_retained_approval_binding,
    observe_retained_assessment_evidence, observe_retained_transition,
    observe_retained_workflow_proposal_identity, recover_settled_live_membership,
};
use step_budget::history_basis;
pub(in crate::domain_computation::primary_graph::application_attempt) use step_budget::WorkflowInstanceObservationPurpose;
pub(in crate::domain_computation::primary_graph) use step_budget::WorkflowStepAllowance;
pub(in crate::domain_computation::primary_graph) use subject::observe_subject;

pub(in crate::domain_computation::primary_graph::application_attempt) struct ObservedWorkflowInstance
{
    pub(in crate::domain_computation::primary_graph::application_attempt) live_membership:
        Option<RelationId>,
    pub(in crate::domain_computation::primary_graph::application_attempt) transitions:
        Vec<ObservedWorkflowTransition>,
    pub(in crate::domain_computation::primary_graph::application_attempt) facts:
        Vec<WorthQueryApplicationObservedFact>,
    pub(in crate::domain_computation::primary_graph::application_attempt) handoff_fact_count: usize,
    pub(in crate::domain_computation::primary_graph::application_attempt) progress_basis:
        WorkflowTransitionProgressBasis,
    pub(in crate::domain_computation::primary_graph::application_attempt) replays:
        crate::domain_computation::primary_graph::workflow::instance::WorkflowTransitionReplayRetention,
    history_materialized: bool,
    history_budget: worth_query_installation::facade::WorthQueryWorkflowHistoryReconstructionBudget,
    purpose: WorkflowInstanceObservationPurpose,
    inherited_steps: u64,
    inherited_evidence_bytes: u64,
    maximum_transitions: usize,
}

pub(in crate::domain_computation::primary_graph::application_attempt) struct ObservedWorkflowTransition
{
    pub(in crate::domain_computation::primary_graph::application_attempt) entity: EntityId,
    pub(in crate::domain_computation::primary_graph::application_attempt) identity: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) settlement:
        SettledWorkflowTransition,
    pub(in crate::domain_computation::primary_graph::application_attempt) assessment_evidence:
        Option<ObservedWorkflowAssessmentEvidence>,
    pub(in crate::domain_computation::primary_graph::application_attempt) operation_outputs:
        Option<history::operation_outputs::ObservedOperationOutputs>,
}

pub(in crate::domain_computation::primary_graph::application_attempt) struct ObservedWorkflowAssessmentEvidence
{
    pub(in crate::domain_computation::primary_graph::application_attempt) entity: EntityId,
    pub(in crate::domain_computation::primary_graph::application_attempt) identity: String,
    /// The bytes it retains against the lineage's evidence ceiling.
    pub(in crate::domain_computation::primary_graph::application_attempt) retained_bytes: u64,
    pub(in crate::domain_computation::primary_graph::application_attempt) query: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) parameter_type: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) result_type: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) binding: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) passing: bool,
    pub(in crate::domain_computation::primary_graph::application_attempt) coverage_identity: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) proposal_identity: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) subject: EntityId,
    pub(in crate::domain_computation::primary_graph::application_attempt) source_identity: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) publication_identity:
        String,
    pub(in crate::domain_computation::primary_graph::application_attempt) output_content_identity:
        String,
    /// Required program revision of this current assessment evidence record.
    pub(in crate::domain_computation::primary_graph::application_attempt) program_revision: String,
}

pub(in crate::domain_computation::primary_graph::application_attempt) fn observe_workflow_instance(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: &PublishedWorkflowInstanceRef,
    subject: EntityId,
    lineage: EntityId,
    // Resumed in place when the instance is a migration successor.
    compiled: &mut CompiledWorkflowDefinition,
    maximum_transitions: usize,
    history_budget: worth_query_installation::facade::WorthQueryWorkflowHistoryReconstructionBudget,
    purpose: WorkflowInstanceObservationPurpose,
) -> Result<ObservedWorkflowInstance, WorthQueryApplicationAttemptDenial> {
    if purpose == WorkflowInstanceObservationPurpose::ReadOnly {
        return Err(denial(
            "read-only history must use its non-progressing owner",
        ));
    }
    let entity = instance.entity_id();
    let kind = layout.instance.entity_kind;
    let mut facts = vec![WorthQueryApplicationObservedFact::Entity {
        entity_id: entity,
        kind,
    }];
    exact(
        execution,
        runtime,
        snapshot,
        entity,
        kind,
        &layout.instance.branch_occurrence,
        AspectValue::UInt64(instance.started_occurrence()),
        &mut facts,
    )?;
    actors::observe(execution, runtime, snapshot, layout, entity, &mut facts)?;
    instance_binding::deny_ended(execution, runtime, snapshot, layout, entity)?;
    exact(
        execution,
        runtime,
        snapshot,
        entity,
        kind,
        &layout.instance.program_revision,
        // Instance truth, not the caller's reference, names the revision it
        // runs under: program adoption may have carried it since start.
        text(compiled.program_revision().to_string()),
        &mut facts,
    )?;
    exact(
        execution,
        runtime,
        snapshot,
        entity,
        kind,
        &layout.instance.definition_content_identity,
        text(instance.definition_content_identity().to_string()),
        &mut facts,
    )?;
    subject::observe_subject(
        execution, runtime, snapshot, layout, entity, subject, &mut facts,
    )?;
    instance_binding::resume_definition(
        execution, runtime, snapshot, layout, instance, compiled, &mut facts,
    )?;
    let inherited_steps = instance_binding::inherited(
        execution,
        runtime,
        snapshot,
        layout,
        entity,
        &layout.instance.inherited_steps,
        &mut facts,
    )?;
    let inherited_evidence_bytes = instance_binding::inherited(
        execution,
        runtime,
        snapshot,
        layout,
        entity,
        &layout.instance.inherited_evidence_bytes,
        &mut facts,
    )?;
    let compiled = &*compiled;
    let definitions = adjacency(
        execution,
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
        execution,
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
        execution,
        snapshot,
        layout.live_instance_lineage_relation,
        entity,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        2,
    )?
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
        execution,
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
    let progress_observation = progression::observe_progress(
        execution, handle, runtime, snapshot, layout, instance, &mut facts,
    )?;
    let handoff_fact_count = facts.len();
    let retained_history_key = live_membership
        .is_none()
        .then(|| progress_observation.retained_key())
        .flatten();
    let progress_observation = if live_membership.is_some() {
        match progression::finish_retained_progress(progress_observation, compiled) {
            Ok((progress_basis, replays)) => {
                let (key, _) = progress_basis.replay_retention();
                handle.with_workflow_instance_progress_mut(key, |retention| {
                    retention.observe_warm_core()
                });
                return Ok(ObservedWorkflowInstance {
                    live_membership,
                    transitions: Vec::new(),
                    facts,
                    handoff_fact_count,
                    progress_basis,
                    replays,
                    history_materialized: false,
                    history_budget,
                    purpose,
                    inherited_steps,
                    inherited_evidence_bytes,
                    maximum_transitions,
                });
            }
            Err(observation) => observation,
        }
    } else {
        progress_observation
    };
    let mut history = history::observe(
        execution,
        runtime,
        snapshot,
        layout,
        entity,
        history_basis(live_membership.is_some(), purpose, inherited_steps),
        maximum_transitions,
        history::HistoryNodes::Compiled(compiled),
        history_budget,
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
        .map(|transition| transition.progress_observation())
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
    let (key, _) = progress_basis.replay_retention();
    handle.with_workflow_instance_progress_mut(key, |retention| {
        retention.observe_history_reconstruction_charge(history.charge_bytes)
    });
    facts.append(&mut history.facts);
    Ok(ObservedWorkflowInstance {
        live_membership,
        transitions: settled_transitions,
        facts,
        handoff_fact_count,
        progress_basis,
        replays,
        history_materialized: true,
        history_budget,
        purpose,
        inherited_steps,
        inherited_evidence_bytes,
        maximum_transitions,
    })
}

fn denial(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
