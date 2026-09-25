use super::{CompiledWorkflowConnectionKind, CompiledWorkflowNodeKind};
use crate::domain_computation::primary_graph::workflow::definition::codec::WorkflowConnectionTag;

impl CompiledWorkflowNodeKind {
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) fn retained_string_bytes(
        &self,
    ) -> usize {
        match self {
            Self::Operation {
                operation,
                input_type,
                ..
            } => operation.len().saturating_add(input_type.len()),
            Self::Assessment {
                query,
                parameter_type,
                result_type,
                binding,
                subject,
            } => query
                .len()
                .saturating_add(parameter_type.len())
                .saturating_add(result_type.len())
                .saturating_add(binding.len())
                .saturating_add(subject.persistence_identity().len()),
            Self::Condition {
                query,
                parameter_type,
                result_type,
                binding,
            } => query
                .len()
                .saturating_add(parameter_type.len())
                .saturating_add(result_type.len())
                .saturating_add(binding.len()),
            Self::Approval {
                capability,
                capability_type,
                operation,
                installed_capability_identity,
            } => capability
                .len()
                .saturating_add(capability_type.len())
                .saturating_add(operation.len())
                .saturating_add(installed_capability_identity.len()),
            Self::EvidenceJoin { policy } => policy.identity().len(),
            Self::Terminal => 0,
        }
    }
}

impl CompiledWorkflowConnectionKind {
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) fn retained_string_bytes(
        &self,
    ) -> usize {
        match self {
            Self::Retry { reason, .. } => reason.len(),
            Self::Control(_) | Self::Data(_) => 0,
        }
    }

    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) fn semantic_identity_material(
        &self,
    ) -> String {
        match self {
            Self::Control(outcome) => WorkflowConnectionTag::control(*outcome)
                .identity()
                .to_owned(),
            Self::Data(flow) => WorkflowConnectionTag::data(*flow).identity().to_owned(),
            Self::Retry {
                trigger,
                reason,
                maximum_attempts,
            } => format!(
                "{}:{}:{}",
                WorkflowConnectionTag::Retry(*trigger).identity(),
                reason,
                maximum_attempts,
            ),
        }
    }
}
