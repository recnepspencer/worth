use worth_query_declaration::facade::application_program::{
    ApplicationProgramRevision, ApplicationWorkflowControlOutcome, ApplicationWorkflowDataFlow,
    ApplicationWorkflowDefinitionContentIdentity,
};
use worth_relational::facade::identity::EntityId;

use super::super::codec::{WorkflowConnectionTag, WorkflowNodeTag};

/// Rebuildable workflow meaning for one exact performed definition revision.
///
/// This phase carries no principal, currentness, reservation, or execution
/// authority. The instance owner must freshly admit every use.
pub(in crate::domain_computation::primary_graph) struct CompiledWorkflowDefinition {
    pub(super) lineage: EntityId,
    pub(super) definition: EntityId,
    pub(super) content_identity: ApplicationWorkflowDefinitionContentIdentity,
    pub(super) program_revision: ApplicationProgramRevision,
    pub(super) start_node: EntityId,
    pub(super) nodes: Box<[CompiledWorkflowNode]>,
    pub(super) connections: Box<[CompiledWorkflowConnection]>,
}

pub(in crate::domain_computation::primary_graph) struct CompiledWorkflowNode {
    pub(super) entity: EntityId,
    pub(super) path: String,
    pub(super) kind: CompiledWorkflowNodeKind,
}

pub(in crate::domain_computation::primary_graph) enum CompiledWorkflowNodeKind {
    Operation {
        operation: String,
        input_type: String,
        requires_workflow_authority: bool,
    },
    Assessment {
        query: String,
        parameter_type: String,
        result_type: String,
        binding: String,
    },
    Condition {
        query: String,
        parameter_type: String,
        result_type: String,
        binding: String,
    },
    Approval {
        capability: String,
        capability_type: String,
        operation: String,
        installed_capability_identity: String,
    },
    EvidenceJoin {
        policy: worth_query_declaration::facade::application_program::ApplicationWorkflowEvidenceJoinPolicy,
    },
    Terminal,
}

pub(in crate::domain_computation::primary_graph) struct CompiledWorkflowConnection {
    pub(super) entity: EntityId,
    pub(super) source: EntityId,
    pub(super) target: EntityId,
    pub(super) kind: CompiledWorkflowConnectionKind,
}

pub(in crate::domain_computation::primary_graph) enum CompiledWorkflowConnectionKind {
    Control(ApplicationWorkflowControlOutcome),
    Data(ApplicationWorkflowDataFlow),
    Retry {
        trigger: ApplicationWorkflowControlOutcome,
        reason: String,
        maximum_attempts: u16,
    },
}

impl CompiledWorkflowNode {
    pub(in crate::domain_computation::primary_graph) const fn entity(&self) -> EntityId {
        self.entity
    }

    pub(in crate::domain_computation::primary_graph) fn path(&self) -> &str {
        &self.path
    }

    pub(in crate::domain_computation::primary_graph) const fn kind(
        &self,
    ) -> &CompiledWorkflowNodeKind {
        &self.kind
    }

    pub(in crate::domain_computation::primary_graph) fn identity_material(&self) -> String {
        let mut fields = vec![
            self.entity.partition_value().to_string(),
            self.entity.local_slot_value().to_string(),
            self.entity.generation_value().to_string(),
        ];
        fields.extend(match &self.kind {
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
            } => vec![
                WorkflowNodeTag::Assessment.identity().to_owned(),
                query.clone(),
                parameter_type.clone(),
                result_type.clone(),
                binding.clone(),
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
    fn identity_material(&self) -> String {
        let kind = match &self.kind {
            CompiledWorkflowConnectionKind::Control(outcome) => {
                WorkflowConnectionTag::control(*outcome)
                    .identity()
                    .to_owned()
            }
            CompiledWorkflowConnectionKind::Data(flow) => {
                WorkflowConnectionTag::data(*flow).identity().to_owned()
            }
            CompiledWorkflowConnectionKind::Retry {
                trigger,
                reason,
                maximum_attempts,
            } => format!(
                "{}:{}:{}",
                WorkflowConnectionTag::Retry(*trigger).identity(),
                reason,
                maximum_attempts,
            ),
        };
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
    pub(in crate::domain_computation::primary_graph) const fn lineage(&self) -> EntityId {
        self.lineage
    }

    pub(in crate::domain_computation::primary_graph) const fn definition(&self) -> EntityId {
        self.definition
    }

    pub(in crate::domain_computation::primary_graph) const fn content_identity(
        &self,
    ) -> &ApplicationWorkflowDefinitionContentIdentity {
        &self.content_identity
    }

    pub(in crate::domain_computation::primary_graph) const fn program_revision(
        &self,
    ) -> &ApplicationProgramRevision {
        &self.program_revision
    }

    pub(in crate::domain_computation::primary_graph) fn start_path(&self) -> &str {
        &self.start().path
    }

    pub(in crate::domain_computation::primary_graph) const fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(in crate::domain_computation::primary_graph) fn start(&self) -> &CompiledWorkflowNode {
        self.nodes
            .iter()
            .find(|node| node.entity == self.start_node)
            .expect("compiled workflow start belongs to its node inventory")
    }

    pub(in crate::domain_computation::primary_graph) fn control_successors(
        &self,
        source: EntityId,
        outcome: ApplicationWorkflowControlOutcome,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.connections
            .iter()
            .filter_map(move |connection| match connection.kind {
                CompiledWorkflowConnectionKind::Control(candidate)
                    if candidate == outcome && connection.source == source =>
                {
                    Some(connection.target)
                }
                _ => None,
            })
            .filter_map(|target| self.nodes.iter().find(|node| node.entity == target))
    }

    pub(in crate::domain_computation::primary_graph) fn retry_successors(
        &self,
        source: EntityId,
        trigger: ApplicationWorkflowControlOutcome,
    ) -> impl Iterator<Item = (&CompiledWorkflowNode, u16)> {
        self.connections
            .iter()
            .filter_map(move |connection| match &connection.kind {
                CompiledWorkflowConnectionKind::Retry {
                    trigger: candidate,
                    maximum_attempts,
                    ..
                } if *candidate == trigger && connection.source == source => {
                    Some((connection.target, *maximum_attempts))
                }
                _ => None,
            })
            .filter_map(|(target, maximum_attempts)| {
                self.nodes
                    .iter()
                    .find(|node| node.entity == target)
                    .map(|node| (node, maximum_attempts))
            })
    }

    pub(in crate::domain_computation::primary_graph) fn required_assessments(
        &self,
        join: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.connections
            .iter()
            .filter_map(move |connection| match connection.kind {
                CompiledWorkflowConnectionKind::Data(
                    ApplicationWorkflowDataFlow::AssessmentEvidence,
                ) if connection.target == join => Some(connection.source),
                _ => None,
            })
            .filter_map(|source| self.nodes.iter().find(|node| node.entity == source))
    }

    pub(in crate::domain_computation::primary_graph) fn approval_authority_targets(
        &self,
        approval: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.connections
            .iter()
            .filter_map(move |connection| match connection.kind {
                CompiledWorkflowConnectionKind::Data(
                    ApplicationWorkflowDataFlow::ApprovalAuthority,
                ) if connection.source == approval => Some(connection.target),
                _ => None,
            })
            .filter_map(|target| self.nodes.iter().find(|node| node.entity == target))
    }

    pub(in crate::domain_computation::primary_graph) fn approval_proposal_sources(
        &self,
        approval: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.data_sources(approval, ApplicationWorkflowDataFlow::ProposalSubject)
    }

    pub(in crate::domain_computation::primary_graph) fn approval_evidence_sources(
        &self,
        approval: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.data_sources(approval, ApplicationWorkflowDataFlow::JoinedEvidence)
    }

    pub(in crate::domain_computation::primary_graph) fn operation_input_sources(
        &self,
        operation: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.data_sources(operation, ApplicationWorkflowDataFlow::OperationInput)
    }

    fn data_sources(
        &self,
        target: EntityId,
        flow: ApplicationWorkflowDataFlow,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.connections
            .iter()
            .filter_map(move |connection| match connection.kind {
                CompiledWorkflowConnectionKind::Data(candidate)
                    if candidate == flow && connection.target == target =>
                {
                    Some(connection.source)
                }
                _ => None,
            })
            .filter_map(|source| self.nodes.iter().find(|node| node.entity == source))
    }

    pub(in crate::domain_computation::primary_graph) fn nodes(
        &self,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.nodes.iter()
    }

    pub(in crate::domain_computation::primary_graph) fn structure_identity_material(
        &self,
    ) -> String {
        let mut material = String::new();
        for node in &self.nodes {
            append_framed(&mut material, &node.path);
            append_framed(&mut material, &node.identity_material());
        }
        for connection in &self.connections {
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

#[cfg(test)]
#[path = "plan/tests.rs"]
mod tests;
