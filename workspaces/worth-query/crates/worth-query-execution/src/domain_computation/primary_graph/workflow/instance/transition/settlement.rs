use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::{AdmittedWorkflowTransition, WorkflowOperationSettlementBasis};
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
    emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<CreatedEntityRef, Error> {
    visit_transition_facts(layout, admitted, outcome, state, None, emit)
}

pub(in crate::domain_computation::primary_graph) fn visit_workflow_operation_transition_facts<
    Schema,
    Operation,
    Input,
    Scope,
    Error,
>(
    layout: &WorthQueryWorkflowLayout,
    admitted: &AdmittedWorkflowTransition<Schema, Operation, Input, Scope>,
    operation_receipt_identity: &[u8; 32],
    emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<CreatedEntityRef, Error> {
    visit_transition_facts(
        layout,
        admitted,
        worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Completed,
        super::super::state::WorkflowInstanceState::Ready,
        Some(operation_receipt_identity),
        emit,
    )
}

pub(in crate::domain_computation::primary_graph) fn visit_workflow_operation_settlement_facts<
    Error,
>(
    layout: &WorthQueryWorkflowLayout,
    basis: &WorkflowOperationSettlementBasis,
    operation_receipt_identity: &[u8; 32],
    emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<CreatedEntityRef, Error> {
    visit_transition_facts_from_basis(
        layout,
        TransitionFactBasis::from_settlement(basis),
        worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Completed,
        super::super::state::WorkflowInstanceState::Ready,
        Some(operation_receipt_identity),
        emit,
    )
}

fn visit_transition_facts<Schema, Operation, Input, Scope, Error>(
    layout: &WorthQueryWorkflowLayout,
    admitted: &AdmittedWorkflowTransition<Schema, Operation, Input, Scope>,
    outcome: worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome,
    state: super::super::state::WorkflowInstanceState,
    operation_receipt_identity: Option<&[u8; 32]>,
    emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<CreatedEntityRef, Error> {
    visit_transition_facts_from_basis(
        layout,
        TransitionFactBasis::from_admitted(admitted),
        outcome,
        state,
        operation_receipt_identity,
        emit,
    )
}

struct TransitionFactBasis<'a> {
    instance: worth_relational::facade::identity::EntityId,
    node: worth_relational::facade::identity::EntityId,
    occurrence: u64,
    live_membership: worth_relational::facade::identity::RelationId,
    retire_live_membership: bool,
    identity: &'a str,
}

impl<'a> TransitionFactBasis<'a> {
    fn from_admitted<Schema, Operation, Input, Scope>(
        admitted: &'a AdmittedWorkflowTransition<Schema, Operation, Input, Scope>,
    ) -> Self {
        Self {
            instance: admitted.instance,
            node: admitted.node,
            occurrence: admitted.occurrence,
            live_membership: admitted.live_membership,
            retire_live_membership: admitted.retire_live_membership,
            identity: &admitted.identity,
        }
    }

    fn from_settlement(basis: &'a WorkflowOperationSettlementBasis) -> Self {
        Self {
            instance: basis.instance,
            node: basis.node,
            occurrence: basis.occurrence,
            live_membership: basis.live_membership,
            retire_live_membership: basis.retire_live_membership,
            identity: &basis.identity,
        }
    }
}

fn visit_transition_facts_from_basis<Error>(
    layout: &WorthQueryWorkflowLayout,
    admitted: TransitionFactBasis<'_>,
    outcome: worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome,
    state: super::super::state::WorkflowInstanceState,
    operation_receipt_identity: Option<&[u8; 32]>,
    mut emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<CreatedEntityRef, Error> {
    let transition = CreatedEntityRef {
        partition_id: admitted.instance.partition_id,
        kind_id: layout.transition.entity_kind,
        client_key: ClientKey::raw("transition"),
    };
    let mut fields = BTreeMap::from([
        (
            layout.transition.identity.clone(),
            AspectValue::String(InternedString::Raw(admitted.identity.to_owned())),
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
    ]);
    if let Some(identity) = operation_receipt_identity {
        fields.insert(
            layout.transition.operation_receipt_identity.clone(),
            AspectValue::String(InternedString::Raw(encode_identity(identity))),
        );
    }
    emit(WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: transition.kind_id,
        key: "transition".to_owned(),
        fields,
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

fn encode_identity(identity: &[u8; 32]) -> String {
    let mut encoded = String::with_capacity(64);
    for byte in identity {
        use std::fmt::Write;
        write!(&mut encoded, "{byte:02x}")
            .expect("writing an operation receipt identity to String cannot fail");
    }
    encoded
}
