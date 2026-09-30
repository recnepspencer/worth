use worth_relational::facade::identity::EntityId;

mod decode;
mod shape;
use decode::decode_kind;
use shape::{empty_node_fields, empty_optional_node_fields, invalid_node};

use super::{observed_bool, observed_optional_text, observed_text, observed_u64};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::{
    definition::{
        codec::WorkflowNodeTag,
        compilation::plan::{
            CompiledWorkflowAssessmentApplicability, CompiledWorkflowNode,
            CompiledWorkflowNodeKind, CompiledWorkflowNodeMeaning,
        },
        CompiledWorkflowCondition,
    },
    schema::WorthQueryWorkflowLayout,
};

pub(super) fn compile_node(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    entity: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<CompiledWorkflowNode, WorthQueryApplicationAttemptDenial> {
    let node = &layout.node;
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: entity,
        kind: node.entity_kind,
    });
    let path = observed_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.path,
        facts,
    )?;
    let kind = observed_u64(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.kind,
        facts,
    )?;
    let member = observed_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.member,
        facts,
    )?;
    let input_type = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.input_type,
        facts,
    )?;
    let parameter_type = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.parameter_type,
        facts,
    )?;
    let result_type = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.result_type,
        facts,
    )?;
    let assessment_binding = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.assessment_binding,
        facts,
    )?;
    let assessment_subject = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.assessment_subject,
        facts,
    )?;
    let assessment_applicability_relation = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.assessment_applicability_relation,
        facts,
    )?;
    let assessment_applicability_from = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.assessment_applicability_from,
        facts,
    )?;
    let assessment_applicability_to = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.assessment_applicability_to,
        facts,
    )?;
    let operation_binding = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.operation_binding,
        facts,
    )?;
    let inbound_origin = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.inbound_origin,
        facts,
    )?;
    let inbound_contract = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.inbound_contract,
        facts,
    )?;
    let inbound_wait = super::observed_optional_u64(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.inbound_wait,
        facts,
    )?;
    let condition_binding = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.condition_binding,
        facts,
    )?;
    let condition_operands = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.condition_operands,
        facts,
    )?;
    let capability_type = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.capability_type,
        facts,
    )?;
    let approval_operation = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.approval_operation,
        facts,
    )?;
    let approval_capability_identity = observed_optional_text(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.approval_capability_identity,
        facts,
    )?;
    let requires_workflow_authority = observed_bool(
        runtime,
        snapshot,
        entity,
        node.entity_kind,
        &node.requires_authority,
        facts,
    )?;
    let kind = decode_kind(
        kind,
        member,
        input_type,
        operation_binding,
        decode::InboundFields {
            origin: inbound_origin,
            contract: inbound_contract,
            wait: inbound_wait,
        },
        parameter_type,
        result_type,
        assessment_binding,
        assessment_subject,
        assessment_applicability_relation,
        assessment_applicability_from,
        assessment_applicability_to,
        condition_binding,
        condition_operands,
        capability_type,
        approval_operation,
        approval_capability_identity,
        requires_workflow_authority,
    )?;
    Ok(CompiledWorkflowNode {
        entity,
        meaning: std::sync::Arc::new(CompiledWorkflowNodeMeaning { path, kind }),
    })
}

#[cfg(test)]
#[path = "node/tests.rs"]
mod tests;
