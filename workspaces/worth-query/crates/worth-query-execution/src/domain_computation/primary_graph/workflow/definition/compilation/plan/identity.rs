use super::super::super::codec::WorkflowNodeTag;
use super::{
    CompiledWorkflowAssessmentApplicability, CompiledWorkflowConnection,
    CompiledWorkflowDefinition, CompiledWorkflowNode, CompiledWorkflowNodeKind,
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
                binding,
                requires_workflow_authority,
            } => vec![
                WorkflowNodeTag::Operation.identity().to_owned(),
                operation.clone(),
                input_type.clone(),
                binding.clone().unwrap_or_default(),
                requires_workflow_authority.to_string(),
            ],
            CompiledWorkflowNodeKind::AwaitInbound { origin, effect, protocol, source_identity, limits, wait } => vec![
                WorkflowNodeTag::AwaitInbound.identity().to_owned(),
                origin.clone(), effect.clone(), protocol.identity().as_str().to_owned(),
                protocol.version().get().to_string(), source_identity.clone(),
                limits.maximum_envelope_bytes.get().to_string(),
                limits.maximum_payload_bytes.get().to_string(),
                limits.maximum_outstanding_dispatch_provenance.get().to_string(),
                limits.maximum_accepted_occurrences.get().to_string(),
                limits.maximum_accepted_bytes.get().to_string(),
                limits.maximum_concurrent_publications.get().to_string(),
                limits.maximum_discovery_work.get().to_string(),
                limits.replay_window_milliseconds.get().to_string(),
                limits.maximum_cleanup_work.get().to_string(),
                match wait {
                    worth_query_declaration::facade::application_program::ApplicationWorkflowInboundWait::UntilInstanceDeadline => "instance-deadline".to_owned(),
                },
            ],
            CompiledWorkflowNodeKind::Assessment {
                query,
                parameter_type,
                result_type,
                binding,
                subject,
                applicability,
            } => vec![
                WorkflowNodeTag::Assessment.identity().to_owned(),
                query.clone(),
                parameter_type.clone(),
                result_type.clone(),
                binding.clone(),
                subject.persistence_identity(),
                match applicability {
                    CompiledWorkflowAssessmentApplicability::Always => "always".to_owned(),
                    CompiledWorkflowAssessmentApplicability::WhenRelatedRelationPresent {
                        relation,
                        from,
                        to,
                    } => format!(
                        "related-relation-present:{}:{relation}:{}:{from}:{}:{to}",
                        relation.len(),
                        from.len(),
                        to.len()
                    ),
                },
            ],
            CompiledWorkflowNodeKind::Condition(condition) => {
                std::iter::once(WorkflowNodeTag::Condition.identity().to_owned())
                    .chain(condition.identity_fields())
                    .collect()
            }
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
