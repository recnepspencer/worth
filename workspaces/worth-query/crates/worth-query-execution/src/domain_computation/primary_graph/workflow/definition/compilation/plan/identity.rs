use super::super::super::codec::WorkflowNodeTag;
use super::{
    CompiledWorkflowConnection, CompiledWorkflowDefinition, CompiledWorkflowNode,
    CompiledWorkflowNodeKind,
};

impl CompiledWorkflowNode {
    pub(in crate::domain_computation::primary_graph) fn identity_material(&self) -> String {
        let mut fields = vec![
            self.entity.partition_value().to_string(),
            self.entity.local_slot_value().to_string(),
            self.entity.generation_value().to_string(),
        ];
        fields.extend(match &self.meaning.kind {
            CompiledWorkflowNodeKind::Operation {
                operation,
                input_type,
                requires_workflow_authority,
            } => vec![
                WorkflowNodeTag::Operation.identity().to_owned(),
                operation.clone(),
                input_type.clone(),
                requires_workflow_authority.to_string(),
            ],
            CompiledWorkflowNodeKind::Assessment {
                query,
                parameter_type,
                result_type,
                binding,
                subject,
            } => vec![
                WorkflowNodeTag::Assessment.identity().to_owned(),
                query.clone(),
                parameter_type.clone(),
                result_type.clone(),
                binding.clone(),
                subject.persistence_identity(),
            ],
            CompiledWorkflowNodeKind::Condition {
                query,
                parameter_type,
                result_type,
                binding,
            } => vec![
                WorkflowNodeTag::Condition.identity().to_owned(),
                query.clone(),
                parameter_type.clone(),
                result_type.clone(),
                binding.clone(),
            ],
            CompiledWorkflowNodeKind::Approval {
                capability,
                capability_type,
                operation,
                installed_capability_identity,
            } => vec![
                WorkflowNodeTag::Approval.identity().to_owned(),
                capability.clone(),
                capability_type.clone(),
                operation.clone(),
                installed_capability_identity.clone(),
            ],
            CompiledWorkflowNodeKind::EvidenceJoin { policy } => vec![
                WorkflowNodeTag::EvidenceJoin.identity().to_owned(),
                policy.identity().to_owned(),
            ],
            CompiledWorkflowNodeKind::Terminal => {
                vec![WorkflowNodeTag::Terminal.identity().to_owned()]
            }
        });
        fields
            .into_iter()
            .fold(String::new(), |mut encoded, field| {
                use std::fmt::Write;
                write!(&mut encoded, "{}:{field}", field.len())
                    .expect("writing workflow node identity to String cannot fail");
                encoded
            })
    }
}

impl CompiledWorkflowConnection {
    pub(super) fn identity_material(&self) -> String {
        let kind = self.kind.semantic_identity_material();
        format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}:{kind}",
            self.entity.partition_value(),
            self.entity.local_slot_value(),
            self.entity.generation_value(),
            self.source.partition_value(),
            self.source.local_slot_value(),
            self.source.generation_value(),
            self.target.partition_value(),
            self.target.local_slot_value(),
            self.target.generation_value(),
            kind = kind,
        )
    }
}

impl CompiledWorkflowDefinition {
    pub(in crate::domain_computation::primary_graph) fn structure_identity_material(
        &self,
    ) -> String {
        let mut material = String::new();
        for node in &self.publication.nodes {
            append_framed(&mut material, node.path());
            append_framed(&mut material, &node.identity_material());
        }
        for connection in &self.publication.connections {
            append_framed(&mut material, &connection.identity_material());
        }
        material
    }
}

fn append_framed(target: &mut String, value: &str) {
    use std::fmt::Write;
    write!(target, "{}:{value}", value.len())
        .expect("writing workflow structure identity to String cannot fail");
}
