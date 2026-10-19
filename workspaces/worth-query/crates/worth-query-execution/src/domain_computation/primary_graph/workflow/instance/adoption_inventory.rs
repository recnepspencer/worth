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
    WORKFLOW_BIRTH_FACT_PROTOCOL_VERSION, WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION,
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
    /// An external operation committed into its owner's custody and no
    /// transition has settled it yet.
    pub(in crate::domain_computation::primary_graph) owner_custody_pending: bool,
}

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct WorkflowInventoriedTransition {
    pub(in crate::domain_computation::primary_graph) node: EntityId,
    pub(in crate::domain_computation::primary_graph) occurrence: u64,
    pub(in crate::domain_computation::primary_graph) outcome: ApplicationWorkflowControlOutcome,
    /// The transition settles a performed operation under its receipt.
    pub(in crate::domain_computation::primary_graph) receipted: bool,
}

struct ReadTransition {
    inventoried: WorkflowInventoriedTransition,
    identity: String,
}

pub(in crate::domain_computation::primary_graph) fn read_live_instances(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    truth: &mut WorkflowAdoptionTruth,
    layout: &WorthQueryWorkflowLayout,
    branch_occurrence: u64,
) -> Result<Vec<WorkflowLiveInstance>, WorkflowAdoptionReadDenial> {
    let mut instances = Vec::new();
    for membership in truth.relations_of_kind(execution, layout.live_instance_lineage_relation)? {
        let instance = membership.source;
        let unreadable = WorkflowAdoptionReadDenial::UnreadableEntity { entity: instance };
        let record = truth.entity(execution, instance, layout.instance.entity_kind)?;
        let protocol = record.u64(execution, &layout.instance.protocol_version)?;
        if !matches!(
            protocol,
            WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION | WORKFLOW_BIRTH_FACT_PROTOCOL_VERSION
        ) {
            return Err(unreadable);
        }
        if record.u64(execution, &layout.instance.branch_occurrence)? != branch_occurrence {
            continue;
        }
        if record.u64(execution, &layout.instance.state)?
            != WorkflowInstanceState::Ready.persisted_tag()
        {
            return Err(unreadable);
        }
        if layout
            .instance
            .start_advancer
            .read_adoption(&record, execution, instance)?
            .is_none()
            || layout
                .instance
                .end_advancer
                .read_adoption(&record, execution, instance)?
                .is_some()
        {
            return Err(unreadable);
        }
        record.text(execution, &layout.instance.program_revision)?;
        record.required_symbol(execution, &layout.instance.started_program_revision)?;
        let owner_custody = record.optional_text(execution, &layout.instance.owner_custody)?;
        if protocol == WORKFLOW_BIRTH_FACT_PROTOCOL_VERSION {
            for locator in [
                &layout.instance.subject_partition,
                &layout.instance.subject_slot,
                &layout.instance.subject_generation,
            ] {
                if !record.is_absent(execution, locator)? {
                    return Err(unreadable);
                }
            }
        }
        let definition =
            truth.single_target(execution, instance, layout.instance_definition_relation)?;
        if protocol == WORKFLOW_BIRTH_FACT_PROTOCOL_VERSION {
            truth.single_target(execution, instance, layout.instance_subject_relation)?;
        }
        let mut transitions = Vec::new();
        let mut owner_custody_pending = owner_custody.is_some();
        for relation in truth.outgoing(execution, instance, layout.instance_transition_relation)? {
            let read = read_transition(execution, truth, layout, relation.target)?;
            if owner_custody.as_deref() == Some(read.identity.as_str()) {
                owner_custody_pending = false;
            }
            transitions.push(read.inventoried);
        }
        let inherits_effects = !truth
            .outgoing(execution, instance, layout.instance_prior_effect_relation)?
            .is_empty();
        instances.push(WorkflowLiveInstance {
            instance,
            lineage: membership.target,
            definition,
            live_membership: membership.relation_id,
            transitions,
            inherits_effects,
            owner_custody_pending,
        });
    }
    instances.sort_unstable_by_key(|live| live.instance);
    Ok(instances)
}

fn read_transition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    truth: &mut WorkflowAdoptionTruth,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
) -> Result<ReadTransition, WorkflowAdoptionReadDenial> {
    let unreadable = WorkflowAdoptionReadDenial::UnreadableEntity { entity: transition };
    let record = truth.entity(execution, transition, layout.transition.entity_kind)?;
    let context = super::super::operation_output::decode_transition_output_context(
        record.value(execution, &layout.transition.protocol_version)?,
        record.value(execution, &layout.transition.output_binding)?,
        record.value(execution, &layout.transition.output_count)?,
    )
    .map_err(|()| unreadable)?;
    let identity = record.text(execution, &layout.transition.identity)?;
    let occurrence = record.u64(execution, &layout.transition.occurrence)?;
    let outcome =
        super::decode_transition_outcome(record.u64(execution, &layout.transition.outcome)?)
            .ok_or(unreadable)?;
    let receipted =
        match record.optional_text(execution, &layout.transition.operation_receipt_identity)? {
            Some(receipt) if super::super::proposal::decode_identity(&receipt).is_some() => true,
            Some(_) => return Err(unreadable),
            None => false,
        };
    // Protocol 3 describes performed output context, even when its output count is zero.
    if context.is_some() && !receipted {
        return Err(unreadable);
    }
    if layout
        .transition
        .advancer
        .read_adoption(&record, execution, transition)?
        .is_none()
        || layout
            .transition
            .executor
            .read_adoption(&record, execution, transition)?
            .is_some()
            != receipted
    {
        return Err(unreadable);
    }
    let node = truth.single_target(execution, transition, layout.transition_node_relation)?;
    Ok(ReadTransition {
        inventoried: WorkflowInventoriedTransition {
            node,
            occurrence,
            outcome,
            receipted,
        },
        identity,
    })
}
