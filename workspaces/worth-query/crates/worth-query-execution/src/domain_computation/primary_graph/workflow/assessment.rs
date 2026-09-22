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

#[path = "assessment/dependency.rs"]
mod dependency;

pub(in crate::domain_computation::primary_graph) struct WorkflowAssessmentEvidenceMeaning {
    pub(in crate::domain_computation::primary_graph) identity: String,
    pub(in crate::domain_computation::primary_graph) producer: String,
    pub(in crate::domain_computation::primary_graph) family: String,
    pub(in crate::domain_computation::primary_graph) query: String,
    pub(in crate::domain_computation::primary_graph) parameter_type: String,
    pub(in crate::domain_computation::primary_graph) result_type: String,
    pub(in crate::domain_computation::primary_graph) binding: String,
    pub(in crate::domain_computation::primary_graph) subject: EntityId,
    pub(in crate::domain_computation::primary_graph) proposal_identity: String,
    pub(in crate::domain_computation::primary_graph) coverage_identity: String,
    pub(in crate::domain_computation::primary_graph) source_identity: String,
    pub(in crate::domain_computation::primary_graph) passing: bool,
    pub(in crate::domain_computation::primary_graph) publication_identity: String,
    pub(in crate::domain_computation::primary_graph) output_content_identity: String,
    pub(in crate::domain_computation::primary_graph) currentness_facts:
        std::sync::Arc<[super::super::WorthQueryApplicationObservedFact]>,
}

pub(in crate::domain_computation::primary_graph) fn visit_workflow_assessment_facts<
    Schema,
    Operation,
    Input,
    Scope,
    Error,
>(
    layout: &WorthQueryWorkflowLayout,
    admitted: &AdmittedWorkflowTransition<Schema, Operation, Input, Scope>,
    meaning: &WorkflowAssessmentEvidenceMeaning,
    mut emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<(CreatedEntityRef, CreatedEntityRef), Error> {
    let transition = visit_workflow_transition_facts(
        layout,
        admitted,
        worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Completed,
        WorkflowInstanceState::Ready,
        &mut emit,
    )?;
    let evidence = CreatedEntityRef {
        partition_id: admitted.instance().partition_id,
        kind_id: layout.assessment_evidence.entity_kind,
        client_key: ClientKey::raw("assessment-evidence"),
    };
    emit(WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: evidence.kind_id,
        key: "assessment-evidence".to_owned(),
        fields: BTreeMap::from([
            (
                layout.assessment_evidence.identity.clone(),
                text(&meaning.identity),
            ),
            (
                layout.assessment_evidence.protocol_version.clone(),
                AspectValue::UInt64(super::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION),
            ),
            (
                layout.assessment_evidence.producer.clone(),
                text(&meaning.producer),
            ),
            (
                layout.assessment_evidence.family.clone(),
                text(&meaning.family),
            ),
            (
                layout.assessment_evidence.query.clone(),
                text(&meaning.query),
            ),
            (
                layout.assessment_evidence.parameter_type.clone(),
                text(&meaning.parameter_type),
            ),
            (
                layout.assessment_evidence.result_type.clone(),
                text(&meaning.result_type),
            ),
            (
                layout.assessment_evidence.binding.clone(),
                text(&meaning.binding),
            ),
            (
                layout.assessment_evidence.subject_partition.clone(),
                AspectValue::UInt64(meaning.subject.partition_value_u64()),
            ),
            (
                layout.assessment_evidence.subject_slot.clone(),
                AspectValue::UInt64(meaning.subject.local_slot_value()),
            ),
            (
                layout.assessment_evidence.subject_generation.clone(),
                AspectValue::UInt64(u64::from(meaning.subject.generation_value())),
            ),
            (
                layout.assessment_evidence.proposal_identity.clone(),
                text(&meaning.proposal_identity),
            ),
            (
                layout.assessment_evidence.coverage_identity.clone(),
                text(&meaning.coverage_identity),
            ),
            (
                layout.assessment_evidence.source_identity.clone(),
                text(&meaning.source_identity),
            ),
            (
                layout.assessment_evidence.passing.clone(),
                AspectValue::Bool(meaning.passing),
            ),
            (
                layout.assessment_evidence.publication_identity.clone(),
                text(&meaning.publication_identity),
            ),
            (
                layout.assessment_evidence.output_content_identity.clone(),
                text(&meaning.output_content_identity),
            ),
        ]),
        partition: WorthQueryApplicationCreationPartition::Context(
            admitted.instance().partition_id,
        ),
    })?;
    dependency::visit_dependency_facts(layout, &evidence, meaning, &mut emit)?;
    emit(WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: layout.transition_assessment_evidence_relation,
        key: "transition-assessment-evidence".to_owned(),
        from: EntityReference::Created(transition.clone()),
        to: EntityReference::Created(evidence.clone()),
    })?;
    Ok((transition, evidence))
}

fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.to_owned()))
}
