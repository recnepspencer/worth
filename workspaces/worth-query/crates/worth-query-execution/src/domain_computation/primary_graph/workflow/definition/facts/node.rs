//! The node record: its kind, member, and the optional fields its kind
//! carries.

use std::collections::BTreeMap;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_program::ApplicationWorkflowNodeKind;
use worth_relational::facade::transactions::CreatedEntityRef;

use super::super::codec::WorkflowNodeTag;
use super::super::{encode_draft, encode_operands};
use super::encoding::{hex, raw_key, text};
use super::{
    WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect,
    WorthQueryWorkflowLayout,
};

pub(super) fn create_node(
    layout: &WorthQueryWorkflowLayout,
    reference: &CreatedEntityRef,
    node: &worth_query_declaration::facade::application_program::ApplicationWorkflowNode,
    assessment_binding: Option<&str>,
    condition_bindings: &[&worth_query_installation::facade::WorthQueryInstalledWorkflowConditionBinding],
    approval_binding: Option<
        &worth_query_installation::facade::WorthQueryInstalledWorkflowApprovalBinding,
    >,
    creation_partition: WorthQueryApplicationCreationPartition,
) -> WorthQueryApplicationRealizedEffect {
    let kind = WorkflowNodeTag::from_declared(node.kind()).persisted();
    let condition_member = match node.kind() {
        ApplicationWorkflowNodeKind::Condition(condition) => encode_draft(condition.draft()),
        _ => String::new(),
    };
    let (member, input_type, parameter_type, result_type, capability_type, requires_authority) =
        match node.kind() {
            ApplicationWorkflowNodeKind::Operation {
                operation,
                requires_workflow_authority,
            } => (
                operation.identifier(),
                Some(operation.input_type().as_str()),
                None,
                None,
                None,
                *requires_workflow_authority,
            ),
            ApplicationWorkflowNodeKind::Assessment(assessment) => (
                assessment.identifier(),
                None,
                Some(assessment.parameter_type().as_str()),
                Some(assessment.result_type().as_str()),
                None,
                false,
            ),
            ApplicationWorkflowNodeKind::Condition(_) => {
                (condition_member.as_str(), None, None, None, None, false)
            }
            ApplicationWorkflowNodeKind::Approval(approval) => (
                approval.identifier(),
                None,
                None,
                None,
                Some(approval.capability_type().as_str()),
                false,
            ),
            ApplicationWorkflowNodeKind::EvidenceJoin(policy) => {
                (policy.identity(), None, None, None, None, false)
            }
            ApplicationWorkflowNodeKind::Terminal => ("", None, None, None, None, false),
        };
    let mut fields = BTreeMap::from([
        (layout.node.path.clone(), text(node.identity().as_str())),
        (layout.node.kind.clone(), AspectValue::UInt64(kind)),
        (layout.node.member.clone(), text(member)),
        (
            layout.node.requires_authority.clone(),
            AspectValue::Bool(requires_authority),
        ),
    ]);
    for (locator, value) in [
        (&layout.node.input_type, input_type),
        (
            &layout.node.operation_binding,
            match node.kind() {
                ApplicationWorkflowNodeKind::Operation { operation, .. } => {
                    operation.binding().map(|(identity, _, _)| identity)
                }
                _ => None,
            },
        ),
        (&layout.node.parameter_type, parameter_type),
        (&layout.node.result_type, result_type),
        (&layout.node.assessment_binding, assessment_binding),
        (&layout.node.capability_type, capability_type),
        (
            &layout.node.approval_operation,
            approval_binding.map(|binding| binding.operation),
        ),
    ] {
        if let Some(value) = value {
            fields.insert(locator.clone(), text(value));
        }
    }
    if let Some(binding) = approval_binding {
        fields.insert(
            layout.node.approval_capability_identity.clone(),
            text(hex(binding.installed_capability_identity)),
        );
    }
    if let ApplicationWorkflowNodeKind::Condition(condition) = node.kind() {
        // Installation admits a definition only when every operand has a
        // binding, so a published condition always records all of them.
        let operands = condition
            .operands()
            .iter()
            .map(|operand| {
                let query = operand.query();
                let binding = condition_bindings
                    .iter()
                    .find(|binding| &*binding.operand == operand.name())
                    .expect("installed condition binds every operand");
                [
                    operand.name(),
                    query.identifier(),
                    query.parameter_type().as_str(),
                    query.result_type().as_str(),
                    binding.binding,
                ]
            })
            .collect::<Vec<_>>();
        fields.insert(
            layout.node.condition_operands.clone(),
            text(encode_operands(operands)),
        );
    }
    if let ApplicationWorkflowNodeKind::Assessment(assessment) = node.kind() {
        fields.insert(
            layout.node.assessment_subject.clone(),
            text(assessment.subject().persistence_identity()),
        );
        if let Some((relation, from, to)) = assessment.applicability().relation() {
            fields.insert(
                layout.node.assessment_applicability_relation.clone(),
                text(relation),
            );
            fields.insert(
                layout.node.assessment_applicability_from.clone(),
                text(from),
            );
            fields.insert(layout.node.assessment_applicability_to.clone(), text(to));
        }
    }
    WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: reference.kind_id,
        key: raw_key(reference),
        fields,
        partition: creation_partition,
    }
}
