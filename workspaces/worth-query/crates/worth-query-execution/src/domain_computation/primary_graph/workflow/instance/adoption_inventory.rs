//! Live workflow instances on one exact branch incarnation, read from owner
//! truth for program adoption.
//!
//! Only instances this incarnation authored are live here. A fork copies its
//! parent's instance facts, but those copies carry the parent's branch
//! occurrence and stay historical until an explicit fork continuation.
//!
//! Custody reads each live instance's whole transition history. Every read is
//! charged to the one adoption work budget, so a long history exhausts the
//! caller's `maximum_selection_work` with a typed denial instead of running
//! unbounded.

use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::{EntityId, RelationId};

use super::super::adoption::{WorkflowAdoptionReadDenial, WorkflowAdoptionTruth};
use super::super::schema::version::{
    WORKFLOW_FACT_PROTOCOL_VERSION, WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION,
};
use super::super::schema::WorthQueryWorkflowLayout;
use super::WorkflowInstanceState;

pub(in crate::domain_computation::primary_graph) struct WorkflowLiveInstance {
    pub(in crate::domain_computation::primary_graph) instance: EntityId,
    pub(in crate::domain_computation::primary_graph) lineage: EntityId,
    pub(in crate::domain_computation::primary_graph) definition: EntityId,
    pub(in crate::domain_computation::primary_graph) live_membership: RelationId,
    pub(in crate::domain_computation::primary_graph) transitions:
        Vec<WorkflowInventoriedTransition>,
    /// A migration successor carries performed effects of an earlier
    /// definition. They make it performed but never settle its approvals.
    pub(in crate::domain_computation::primary_graph) inherits_effects: bool,
}

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct WorkflowInventoriedTransition {
    pub(in crate::domain_computation::primary_graph) node: EntityId,
    pub(in crate::domain_computation::primary_graph) occurrence: u64,
    pub(in crate::domain_computation::primary_graph) outcome: ApplicationWorkflowControlOutcome,
    /// The transition settles a performed operation under its receipt.
    pub(in crate::domain_computation::primary_graph) receipted: bool,
}

pub(in crate::domain_computation::primary_graph) fn read_live_instances(
    truth: &mut WorkflowAdoptionTruth<'_>,
    layout: &WorthQueryWorkflowLayout,
    branch_occurrence: u64,
) -> Result<Vec<WorkflowLiveInstance>, WorkflowAdoptionReadDenial> {
    let mut instances = Vec::new();
    for membership in truth.relations_of_kind(layout.live_instance_lineage_relation)? {
        let instance = membership.source;
        let unreadable = WorkflowAdoptionReadDenial::UnreadableEntity { entity: instance };
        let record = truth.entity(instance, layout.instance.entity_kind)?;
        if record.u64(&layout.instance.protocol_version)? != WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION
        {
            return Err(unreadable);
        }
        if record.u64(&layout.instance.branch_occurrence)? != branch_occurrence {
            continue;
        }
        if record.u64(&layout.instance.state)? != WorkflowInstanceState::Ready.persisted_tag() {
            return Err(unreadable);
        }
        record.text(&layout.instance.program_revision)?;
        let definition = truth.single_target(instance, layout.instance_definition_relation)?;
        let mut transitions = Vec::new();
        for relation in truth.outgoing(instance, layout.instance_transition_relation)? {
            transitions.push(read_transition(truth, layout, relation.target)?);
        }
        let inherits_effects = !truth
            .outgoing(instance, layout.instance_prior_effect_relation)?
            .is_empty();
        instances.push(WorkflowLiveInstance {
            instance,
            lineage: membership.target,
            definition,
            live_membership: membership.relation_id,
            transitions,
            inherits_effects,
        });
    }
    instances.sort_unstable_by_key(|live| live.instance);
    Ok(instances)
}

fn read_transition(
    truth: &mut WorkflowAdoptionTruth<'_>,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
) -> Result<WorkflowInventoriedTransition, WorkflowAdoptionReadDenial> {
    let unreadable = WorkflowAdoptionReadDenial::UnreadableEntity { entity: transition };
    let record = truth.entity(transition, layout.transition.entity_kind)?;
    if record.u64(&layout.transition.protocol_version)? != WORKFLOW_FACT_PROTOCOL_VERSION {
        return Err(unreadable);
    }
    let occurrence = record.u64(&layout.transition.occurrence)?;
    let outcome = super::decode_transition_outcome(record.u64(&layout.transition.outcome)?)
        .ok_or(unreadable)?;
    let receipted = record
        .optional_text(&layout.transition.operation_receipt_identity)?
        .is_some();
    let node = truth.single_target(transition, layout.transition_node_relation)?;
    Ok(WorkflowInventoriedTransition {
        node,
        occurrence,
        outcome,
        receipted,
    })
}
