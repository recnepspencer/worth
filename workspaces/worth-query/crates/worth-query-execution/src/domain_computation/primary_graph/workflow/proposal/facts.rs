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
        (
            layout.proposal.coverage_count.clone(),
            AspectValue::UInt64(u64::try_from(proposal.coverages.len()).unwrap_or(u64::MAX)),
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
    for (index, coverage) in proposal.coverages.iter().enumerate() {
        let key = format!("proposal-coverage-{index}");
        let coverage_entity = CreatedEntityRef {
            partition_id: instance.partition_id,
            kind_id: layout.proposal_coverage.entity_kind,
            client_key: ClientKey::raw(key.clone()),
        };
        emit(WorthQueryApplicationRealizedEffect::CreateEntity {
            kind: coverage_entity.kind_id,
            key: key.clone(),
            fields: BTreeMap::from([
                (
                    layout.proposal_coverage.identity.clone(),
                    text(coverage.identity.clone()),
                ),
                (
                    layout.proposal_coverage.selector.clone(),
                    text(coverage.selector.persistence_identity()),
                ),
                (
                    layout.proposal_coverage.subject_partition.clone(),
                    AspectValue::UInt64(coverage.subject.partition_value_u64()),
                ),
                (
                    layout.proposal_coverage.subject_slot.clone(),
                    AspectValue::UInt64(coverage.subject.local_slot_value()),
                ),
                (
                    layout.proposal_coverage.subject_generation.clone(),
                    AspectValue::UInt64(u64::from(coverage.subject.generation_value())),
                ),
            ]),
            partition: WorthQueryApplicationCreationPartition::Context(instance.partition_id),
        })?;
        emit(WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: layout.proposal_coverage_relation,
            key: format!("proposal-coverage-relation-{index}"),
            from: EntityReference::Created(proposal_entity.clone()),
            to: EntityReference::Created(coverage_entity),
        })?;
    }
    Ok((transition, proposal_entity))
}

fn text(value: String) -> AspectValue {
    AspectValue::String(InternedString::Raw(value))
}
