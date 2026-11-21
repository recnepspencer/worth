use crate::domain_computation::primary_graph::PrimaryGraphHandleWorkflowExt;
pub mod ended_admission;
pub mod read_dependencies;
pub mod read_fields;
pub mod recorded_actor;
pub use recorded_actor::{
    original_parent, original_scope, original_transition, recorded_approval,
};
pub mod actors;
pub mod governed_start_path;
pub mod transition_roles;
pub use actors::observe as observe_instance_roles;
pub use transition_roles::observe as observe_transition_roles;
pub mod owner_custody;
pub mod read_only;
pub mod retained_history;
pub use read_only::ReadOnlyWorkflowHistory;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::{EntityId, RelationId};

pub use super::{
    observe_adjacency, observe_field_value, PublishedWorkflowInstanceRef,
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition;
use crate::domain_computation::primary_graph::workflow::instance::{
    SettledWorkflowTransition, WorkflowInstanceState, WorkflowTransitionProgressBasis,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

pub mod completed_source;
pub mod evidence_budget;
pub mod history;
pub(super) use history::observe_ended_history;
pub mod instance_binding;
pub mod progression;
pub mod settlement;
pub mod step_budget;
pub mod subject;
pub use instance_binding::{adjacency, adjacency_with_kind, exact, exact_u64, text};

pub(super) use settlement::{
    observe_evidence_dependencies, observe_retained_approval_binding,
    observe_retained_assessment_evidence, observe_retained_transition,
    observe_retained_workflow_proposal_identity, recover_settled_live_membership,
};
use step_budget::history_basis;
pub use step_budget::WorkflowInstanceObservationPurpose;
pub use step_budget::WorkflowStepAllowance;
pub use subject::observe_subject;

pub struct ObservedWorkflowInstance
{
    pub live_membership:
        Option<RelationId>,
    pub transitions:
        Vec<ObservedWorkflowTransition>,
    pub facts:
        Vec<WorthQueryApplicationObservedFact>,
    pub handoff_fact_count: usize,
    pub progress_basis:
        WorkflowTransitionProgressBasis,
    pub replays:
        crate::domain_computation::primary_graph::workflow::instance::WorkflowTransitionReplayRetention,
    history_materialized: bool,
    history_budget: worth_query_installation::facade::WorthQueryWorkflowHistoryReconstructionBudget,
    purpose: WorkflowInstanceObservationPurpose,
    inherited_steps: u64,
    inherited_evidence_bytes: u64,
    maximum_transitions: usize,
}

pub struct ObservedWorkflowTransition
{
    pub(super) captured_inputs: Option<std::sync::Arc<crate::domain_computation::primary_graph::workflow::captured_input::WorkflowCapturedInputSupport>>,
    pub entity: EntityId,
    pub identity: String,
    pub settlement:
        SettledWorkflowTransition,
    pub assessment_evidence:
        Option<ObservedWorkflowAssessmentEvidence>,
    pub operation_outputs:
        Option<history::operation_outputs::ObservedOperationOutputs>,
}

pub struct ObservedWorkflowAssessmentEvidence {
    pub entity: EntityId,
    pub identity: String,
    /// The bytes it retains against the lineage's evidence ceiling.
    pub retained_bytes: u64,
    pub query: String,
    pub parameter_type: String,
    pub result_type: String,
    pub binding: String,
    pub passing: bool,
    pub coverage_identity: String,
    pub proposal_identity: String,
    pub subject: EntityId,
    pub source_identity: String,
    pub publication_identity:
        String,
    pub output_content_identity:
        String,
    /// Required program revision of this current assessment evidence record.
    pub program_revision: String,
}

pub fn observe_workflow_instance(
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

pub fn denial(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}

pub mod history_projection;

