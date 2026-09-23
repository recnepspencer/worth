use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::WorkflowProposalMeaning;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::primary_graph::workflow::{
    instance::{
        visit_workflow_transition_facts, AdmittedWorkflowTransition, WorkflowInstanceState,
    },
    schema::WorthQueryWorkflowLayout,
};

pub(in crate::domain_computation::primary_graph) fn visit_workflow_proposal_facts<
    Schema,
    Operation,
    Input,
    Scope,
    Error,
>(
    layout: &WorthQueryWorkflowLayout,
    admitted: &AdmittedWorkflowTransition<Schema, Operation, Input, Scope>,
    proposal: &WorkflowProposalMeaning,
    mut emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<(CreatedEntityRef, CreatedEntityRef), Error> {
    let transition = visit_workflow_transition_facts(
        layout,
        admitted,
        ApplicationWorkflowControlOutcome::Completed,
        WorkflowInstanceState::Ready,
        &mut emit,
    )?;
    let instance = admitted.instance();
    let proposal_entity = CreatedEntityRef {
        partition_id: instance.partition_id,
        kind_id: layout.proposal.entity_kind,
        client_key: ClientKey::raw("proposal"),
    };
    let mut fields = BTreeMap::from([
        (
            layout.proposal.identity.clone(),
            text(proposal.identity.clone()),
        ),
        (
            layout.proposal.protocol_version.clone(),
            AspectValue::UInt64(
                crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
            ),
        ),
        (
            layout.proposal.operation.clone(),
            text(proposal.operation.clone()),
        ),
        (
            layout.proposal.input_type.clone(),
            text(proposal.input_type.clone()),
        ),
        (
            layout.proposal.input_identity.clone(),
            text(proposal.input_identity.clone()),
        ),
        (
            layout.proposal.node_path.clone(),
            text(proposal.node_path.clone()),
        ),
    ]);
    if let Some(source_identity) = &proposal.source_identity {
        fields.insert(
            layout.proposal.source_identity.clone(),
            text(source_identity.clone()),
        );
    }
    emit(WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: proposal_entity.kind_id,
        key: "proposal".to_owned(),
        fields,
        partition: WorthQueryApplicationCreationPartition::Context(instance.partition_id),
    })?;
    emit(WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: layout.transition_proposal_relation,
        key: "transition-proposal".to_owned(),
        from: EntityReference::Created(transition.clone()),
        to: EntityReference::Created(proposal_entity.clone()),
    })?;
    Ok((transition, proposal_entity))
}

fn text(value: String) -> AspectValue {
    AspectValue::String(InternedString::Raw(value))
}
