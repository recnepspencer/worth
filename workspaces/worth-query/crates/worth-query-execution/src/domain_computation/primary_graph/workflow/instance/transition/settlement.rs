use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::AdmittedWorkflowTransition;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

pub(in crate::domain_computation::primary_graph) fn visit_terminal_transition_facts<
    Schema,
    Operation,
    Input,
    Scope,
    Error,
>(
    layout: &WorthQueryWorkflowLayout,
    admitted: &AdmittedWorkflowTransition<Schema, Operation, Input, Scope>,
    emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<CreatedEntityRef, Error> {
    visit_workflow_transition_facts(
        layout,
        admitted,
        worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Completed,
        super::super::state::WorkflowInstanceState::Completed,
        emit,
    )
}

pub(in crate::domain_computation::primary_graph) fn visit_workflow_transition_facts<
    Schema,
    Operation,
    Input,
    Scope,
    Error,
>(
    layout: &WorthQueryWorkflowLayout,
    admitted: &AdmittedWorkflowTransition<Schema, Operation, Input, Scope>,
    outcome: worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome,
    state: super::super::state::WorkflowInstanceState,
    mut emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<CreatedEntityRef, Error> {
    let transition = CreatedEntityRef {
        partition_id: admitted.instance.partition_id,
        kind_id: layout.transition.entity_kind,
        client_key: ClientKey::raw("transition"),
    };
    emit(WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: transition.kind_id,
        key: "transition".to_owned(),
        fields: BTreeMap::from([
            (
                layout.transition.identity.clone(),
                AspectValue::String(InternedString::Raw(admitted.identity.clone())),
            ),
            (
                layout.transition.protocol_version.clone(),
                AspectValue::UInt64(
                    super::super::super::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
                ),
            ),
            (
                layout.transition.occurrence.clone(),
                AspectValue::UInt64(admitted.occurrence),
            ),
            (
                layout.transition.outcome.clone(),
                AspectValue::UInt64(super::encode_transition_outcome(outcome)),
            ),
            (
                layout.transition.live_membership_partition.clone(),
                AspectValue::UInt64(admitted.live_membership.partition_value_u64()),
            ),
            (
                layout.transition.live_membership_slot.clone(),
                AspectValue::UInt64(admitted.live_membership.local_slot_value()),
            ),
            (
                layout.transition.live_membership_generation.clone(),
                AspectValue::UInt64(u64::from(admitted.live_membership.generation_value())),
            ),
        ]),
        partition: WorthQueryApplicationCreationPartition::Context(admitted.instance.partition_id),
    })?;
    emit(WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: layout.instance_transition_relation,
        key: "instance-transition".to_owned(),
        from: EntityReference::Existing(admitted.instance),
        to: EntityReference::Created(transition.clone()),
    })?;
    emit(WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: layout.transition_node_relation,
        key: "transition-node".to_owned(),
        from: EntityReference::Created(transition.clone()),
        to: EntityReference::Existing(admitted.node),
    })?;
    emit(WorthQueryApplicationRealizedEffect::UpdateEntity {
        entity: "workflow-instance".to_owned(),
        entity_id: admitted.instance,
        fields: BTreeMap::from([(
            layout.instance.state.clone(),
            AspectValue::UInt64(state.persisted_tag()),
        )]),
    })?;
    if admitted.retire_live_membership {
        emit(WorthQueryApplicationRealizedEffect::DeleteRelation {
            relation_id: admitted.live_membership,
        })?;
    }
    Ok(transition)
}
