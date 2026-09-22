use std::sync::Arc;

use worth_query_declaration::facade::application_program::{
    ApplicationProgramRevision, ApplicationWorkflowControlOutcome, ApplicationWorkflowDataFlow,
    ApplicationWorkflowDefinitionContentIdentity,
};
use worth_relational::facade::identity::EntityId;

mod coverage;
mod dispatch;
mod identity;
mod retained_bytes;

pub(super) use dispatch::CompiledWorkflowDispatch;

/// Rebuildable workflow meaning for one exact performed definition revision.
///
/// This phase carries no principal, currentness, reservation, or execution
/// authority. The instance owner must freshly admit every use.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct CompiledWorkflowDefinition {
    pub(super) publication: Arc<CompiledWorkflowPublicationPlan>,
    pub(super) content_identity: ApplicationWorkflowDefinitionContentIdentity,
    pub(super) program_revision: ApplicationProgramRevision,
}

pub(super) struct CompiledWorkflowPublicationPlan {
    pub(super) lineage: EntityId,
    pub(super) definition: EntityId,
    pub(super) start: usize,
    pub(super) nodes: Box<[CompiledWorkflowNode]>,
    pub(super) connections: Box<[CompiledWorkflowConnection]>,
    pub(super) node_ordinals: Arc<[(EntityId, usize)]>,
    pub(super) dispatch: Arc<CompiledWorkflowDispatch>,
}

pub(in crate::domain_computation::primary_graph) struct CompiledWorkflowNode {
    pub(super) entity: EntityId,
    pub(super) meaning: Arc<CompiledWorkflowNodeMeaning>,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct CompiledWorkflowNodeMeaning {
    pub(super) path: String,
    pub(super) kind: CompiledWorkflowNodeKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
        subject: worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector,
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
    pub(super) kind: Arc<CompiledWorkflowConnectionKind>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum CompiledWorkflowConnectionKind {
    Control(ApplicationWorkflowControlOutcome),
    Data(ApplicationWorkflowDataFlow),
    Retry {
        trigger: ApplicationWorkflowControlOutcome,
        reason: String,
        maximum_attempts: u16,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct CompiledWorkflowSemanticConnection {
    pub(super) source: usize,
    pub(super) target: usize,
    pub(super) kind: Arc<CompiledWorkflowConnectionKind>,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct CompiledWorkflowSemanticPlan {
    pub(super) start: usize,
    pub(super) nodes: Box<[Arc<CompiledWorkflowNodeMeaning>]>,
    pub(super) connections: Box<[CompiledWorkflowSemanticConnection]>,
    pub(super) dispatch: Arc<CompiledWorkflowDispatch>,
    pub(super) retained_bytes: usize,
}

impl CompiledWorkflowNode {
    pub(in crate::domain_computation::primary_graph) const fn entity(&self) -> EntityId {
        self.entity
    }

    pub(in crate::domain_computation::primary_graph) fn path(&self) -> &str {
        &self.meaning.path
    }

    pub(in crate::domain_computation::primary_graph) fn kind(&self) -> &CompiledWorkflowNodeKind {
        &self.meaning.kind
    }
}

impl CompiledWorkflowDefinition {
    pub(in crate::domain_computation::primary_graph) fn lineage(&self) -> EntityId {
        self.publication.lineage
    }

    pub(in crate::domain_computation::primary_graph) fn definition(&self) -> EntityId {
        self.publication.definition
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
        self.start().path()
    }

    pub(in crate::domain_computation::primary_graph) fn node_count(&self) -> usize {
        self.publication.nodes.len()
    }

    pub(in crate::domain_computation::primary_graph) fn start(&self) -> &CompiledWorkflowNode {
        &self.publication.nodes[self.publication.start]
    }

    pub(in crate::domain_computation::primary_graph) fn node(
        &self,
        entity: EntityId,
    ) -> Option<&CompiledWorkflowNode> {
        self.node_ordinal(entity)
            .map(|ordinal| &self.publication.nodes[ordinal])
    }

    pub(in crate::domain_computation::primary_graph) fn nodes_with_path(
        &self,
        path: &str,
    ) -> &[CompiledWorkflowNode] {
        let first = self
            .publication
            .nodes
            .partition_point(|node| node.path() < path);
        let count = self.publication.nodes[first..].partition_point(|node| node.path() == path);
        &self.publication.nodes[first..first + count]
    }

    pub(in crate::domain_computation::primary_graph) fn control_successors(
        &self,
        source: EntityId,
        outcome: ApplicationWorkflowControlOutcome,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.outgoing(source).filter_map(move |edge| {
            match self.publication.connections[edge.connection].kind.as_ref() {
                CompiledWorkflowConnectionKind::Control(candidate) if *candidate == outcome => {
                    Some(&self.publication.nodes[edge.peer])
                }
                _ => None,
            }
        })
    }

    pub(in crate::domain_computation::primary_graph) fn retry_successors(
        &self,
        source: EntityId,
        trigger: ApplicationWorkflowControlOutcome,
    ) -> impl Iterator<Item = (&CompiledWorkflowNode, u16)> {
        self.outgoing(source).filter_map(move |edge| {
            match self.publication.connections[edge.connection].kind.as_ref() {
                CompiledWorkflowConnectionKind::Retry {
                    trigger: candidate,
                    maximum_attempts,
                    ..
                } if *candidate == trigger => {
                    Some((&self.publication.nodes[edge.peer], *maximum_attempts))
                }
                _ => None,
            }
        })
    }

    pub(in crate::domain_computation::primary_graph) fn required_assessments(
        &self,
        join: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.incoming(join).filter_map(move |edge| {
            match self.publication.connections[edge.connection].kind.as_ref() {
                CompiledWorkflowConnectionKind::Data(
                    ApplicationWorkflowDataFlow::AssessmentEvidence,
                ) => Some(&self.publication.nodes[edge.peer]),
                _ => None,
            }
        })
    }

    pub(in crate::domain_computation::primary_graph) fn assessment_subject_sources(
        &self,
        assessment: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.data_sources(assessment, ApplicationWorkflowDataFlow::AssessmentSubject)
    }

    pub(in crate::domain_computation::primary_graph) fn approval_authority_targets(
        &self,
        approval: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.data_targets(approval, ApplicationWorkflowDataFlow::ApprovalAuthority)
    }

    fn data_targets(
        &self,
        source: EntityId,
        flow: ApplicationWorkflowDataFlow,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.outgoing(source).filter_map(move |edge| {
            match self.publication.connections[edge.connection].kind.as_ref() {
                CompiledWorkflowConnectionKind::Data(candidate) if *candidate == flow => {
                    Some(&self.publication.nodes[edge.peer])
                }
                _ => None,
            }
        })
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
        self.incoming(target).filter_map(move |edge| {
            match self.publication.connections[edge.connection].kind.as_ref() {
                CompiledWorkflowConnectionKind::Data(candidate) if *candidate == flow => {
                    Some(&self.publication.nodes[edge.peer])
                }
                _ => None,
            }
        })
    }

    fn node_ordinal(&self, entity: EntityId) -> Option<usize> {
        self.publication
            .node_ordinals
            .binary_search_by_key(&entity, |(candidate, _)| *candidate)
            .ok()
            .map(|index| self.publication.node_ordinals[index].1)
    }

    fn outgoing(
        &self,
        source: EntityId,
    ) -> impl Iterator<Item = &dispatch::CompiledWorkflowDispatchEdge> {
        self.node_ordinal(source)
            .into_iter()
            .flat_map(|ordinal| self.publication.dispatch.outgoing(ordinal))
    }

    fn incoming(
        &self,
        target: EntityId,
    ) -> impl Iterator<Item = &dispatch::CompiledWorkflowDispatchEdge> {
        self.node_ordinal(target)
            .into_iter()
            .flat_map(|ordinal| self.publication.dispatch.incoming(ordinal))
    }
}

#[cfg(test)]
#[path = "plan/tests.rs"]
mod tests;
