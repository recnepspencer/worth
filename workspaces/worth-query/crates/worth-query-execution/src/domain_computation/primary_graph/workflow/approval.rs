use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::{
    instance::{
        visit_workflow_transition_facts, AdmittedWorkflowTransition, WorkflowInstanceState,
    },
    schema::WorthQueryWorkflowLayout,
};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect,
};

pub(in crate::domain_computation::primary_graph) struct WorkflowApprovalMeaning {
    pub identity: [u8; 32],
    pub identity_text: String,
    pub instance_identity: String,
    pub definition_content_identity: String,
    pub program_revision: String,
    pub proposal: EntityId,
    pub evidence: Box<[EntityId]>,
    pub approval_operation: String,
    pub target_operation: String,
    pub installed_capability_identity: String,
    pub approver: EntityId,
    pub grant: EntityId,
    pub authorization_decision: [u8; 32],
    pub action: String,
    pub purpose: String,
    pub timeline: &'static str,
    pub sampled_value: String,
    pub expiry: worth_foundational::facade::AspectValue,
    pub scope: EntityId,
    pub decision: super::super::application_attempt::WorkflowApprovalDecision,
}

pub(in crate::domain_computation::primary_graph) fn workflow_approval_fields(
    layout: &WorthQueryWorkflowLayout,
    meaning: &WorkflowApprovalMeaning,
) -> BTreeMap<worth_foundational::facade::AspectFieldLocator, AspectValue> {
    let mut fields = BTreeMap::from([
        (
            layout.approval.identity.clone(),
            text(&meaning.identity_text),
        ),
        (
            layout.approval.protocol_version.clone(),
            AspectValue::UInt64(super::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION),
        ),
        (
            layout.approval.instance_identity.clone(),
            text(&meaning.instance_identity),
        ),
        (
            layout.approval.definition_content_identity.clone(),
            text(&meaning.definition_content_identity),
        ),
        (
            layout.approval.program_revision.clone(),
            text(&meaning.program_revision),
        ),
        (
            layout.approval.decision.clone(),
            text(match meaning.decision {
                super::super::application_attempt::WorkflowApprovalDecision::Approve => "approve",
                super::super::application_attempt::WorkflowApprovalDecision::Reject => "reject",
            }),
        ),
        (
            layout.approval.approval_operation.clone(),
            text(&meaning.approval_operation),
        ),
        (
            layout.approval.target_operation.clone(),
            text(&meaning.target_operation),
        ),
        (
            layout.approval.installed_capability_identity.clone(),
            text(&meaning.installed_capability_identity),
        ),
        (
            layout.approval.authorization_decision.clone(),
            text(hex(meaning.authorization_decision)),
        ),
        (layout.approval.action.clone(), text(&meaning.action)),
        (layout.approval.purpose.clone(), text(&meaning.purpose)),
        (
            layout.approval.validity_timeline.clone(),
            text(meaning.timeline),
        ),
        (
            layout.approval.authorization_sample.clone(),
            text(&meaning.sampled_value),
        ),
        (layout.approval.expiry.clone(), meaning.expiry.clone()),
    ]);
    for (entity, partition, slot, generation) in [
        (
            meaning.approver,
            &layout.approval.approver_partition,
            &layout.approval.approver_slot,
            &layout.approval.approver_generation,
        ),
        (
            meaning.scope,
            &layout.approval.scope_partition,
            &layout.approval.scope_slot,
            &layout.approval.scope_generation,
        ),
        (
            meaning.grant,
            &layout.approval.grant_partition,
            &layout.approval.grant_slot,
            &layout.approval.grant_generation,
        ),
    ] {
        fields.insert(
            partition.clone(),
            AspectValue::UInt64(entity.partition_value_u64()),
        );
        fields.insert(slot.clone(), AspectValue::UInt64(entity.local_slot_value()));
        fields.insert(
            generation.clone(),
            AspectValue::UInt64(u64::from(entity.generation_value())),
        );
    }
    fields
}

pub(in crate::domain_computation::primary_graph) fn visit_workflow_approval_facts<
    Schema,
    Operation,
    Input,
    Scope,
    Error,
>(
    layout: &WorthQueryWorkflowLayout,
    admitted: &AdmittedWorkflowTransition<Schema, Operation, Input, Scope>,
    meaning: &WorkflowApprovalMeaning,
    mut emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<(CreatedEntityRef, CreatedEntityRef), Error> {
    let transition = visit_workflow_transition_facts(
        layout,
        admitted,
        meaning.decision.outcome(),
        WorkflowInstanceState::Ready,
        &mut emit,
    )?;
    let approval = CreatedEntityRef {
        partition_id: admitted.instance().partition_id,
        kind_id: layout.approval.entity_kind,
        client_key: ClientKey::raw("approval"),
    };
    let fields = workflow_approval_fields(layout, meaning);
    emit(WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: approval.kind_id,
        key: "approval".to_owned(),
        fields,
        partition: WorthQueryApplicationCreationPartition::Context(
            admitted.instance().partition_id,
        ),
    })?;
    emit(WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: layout.transition_approval_relation,
        key: "transition-approval".to_owned(),
        from: EntityReference::Created(transition.clone()),
        to: EntityReference::Created(approval.clone()),
    })?;
    emit(WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: layout.approval_proposal_relation,
        key: "approval-proposal".to_owned(),
        from: EntityReference::Created(approval.clone()),
        to: EntityReference::Existing(meaning.proposal),
    })?;
    for (index, evidence) in meaning.evidence.iter().enumerate() {
        emit(WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: layout.approval_evidence_relation,
            key: format!("approval-evidence-{index}"),
            from: EntityReference::Created(approval.clone()),
            to: EntityReference::Existing(*evidence),
        })?;
    }
    Ok((transition, approval))
}

fn text(value: impl Into<String>) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.into()))
}

fn hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write;
    let mut text = String::with_capacity(64);
    for byte in bytes {
        write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}
