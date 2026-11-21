use std::sync::Arc;

use worth_query_declaration::facade::application_program::{
    ApplicationProgramRevision, ApplicationWorkflowControlOutcome, ApplicationWorkflowDataFlow,
    ApplicationWorkflowDefinitionContentIdentity,
};
use worth_relational::facade::identity::EntityId;

pub mod back_action;
pub mod coverage;
pub mod dispatch;
pub mod global_actions;
pub mod identity;
pub mod resume;
pub mod retained_bytes;

pub(super) use dispatch::CompiledWorkflowDispatch;

/// Rebuildable workflow meaning for one exact performed definition revision.
///
/// This phase carries no principal, currentness, reservation, or execution
/// authority. The instance owner must freshly admit every use.
#[derive(Clone)]
pub struct CompiledWorkflowDefinition {
    pub(super) publication: Arc<CompiledWorkflowPublicationPlan>,
    pub(super) content_identity: ApplicationWorkflowDefinitionContentIdentity,
    pub(super) program_revision: ApplicationProgramRevision,
    /// A migration successor's first node; see [`Self::resumed_at`].
    pub(super) resume: Option<usize>,
}

pub struct CompiledWorkflowPublicationPlan {
    pub(super) lineage: EntityId,
    pub(super) definition: EntityId,
    pub(super) start: usize,
    pub(super) nodes: Box<[CompiledWorkflowNode]>,
    pub(super) connections: Box<[CompiledWorkflowConnection]>,
    pub(super) global_actions:
        Arc<[worth_query_installation::facade::WorthQueryWorkflowGlobalActionDependency]>,
    pub(super) node_ordinals: Arc<[(EntityId, usize)]>,
    pub(super) dispatch: Arc<CompiledWorkflowDispatch>,
}

pub struct CompiledWorkflowNode {
    pub(super) entity: EntityId,
    pub(super) meaning: Arc<CompiledWorkflowNodeMeaning>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompiledWorkflowNodeMeaning {
    pub(super) path: String,
    pub(super) kind: CompiledWorkflowNodeKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompiledWorkflowNodeKind {
    Operation {
        operation: String,
        input_type: String,
        binding: Option<String>,
        requires_workflow_authority: bool,
        workflow_execution_requirements: worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements,
    },
    AwaitInbound {
        origin: String,
        effect: String,
        protocol: worth_query_declaration::facade::application_schema::ApplicationInboundOccurrenceProtocol,
        source_identity: String,
        limits: worth_query_declaration::facade::application_schema::ApplicationInboundOccurrenceLimits,
        wait: worth_query_declaration::facade::application_program::ApplicationWorkflowInboundWait,
    },
    Assessment {
        query: String,
        parameter_type: String,
        result_type: String,
        binding: String,
        subject: worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector,
        applicability: CompiledWorkflowAssessmentApplicability,
    },
    Condition(super::super::CompiledWorkflowCondition),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompiledWorkflowAssessmentApplicability {
    Always,
    WhenRelatedRelationPresent {
        relation: String,
        from: String,
        to: String,
    },
}

pub struct CompiledWorkflowConnection {
    pub(super) entity: EntityId,
    pub(super) source: EntityId,
    pub(super) target: EntityId,
    pub(super) kind: Arc<CompiledWorkflowConnectionKind>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompiledWorkflowConnectionKind {
    Control(ApplicationWorkflowControlOutcome),
    Data(ApplicationWorkflowDataFlow),
    BackAction {
        operation: String,
        input_type: String,
        binding: String,
        requirements: worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements,
    },
    Retry {
        trigger: ApplicationWorkflowControlOutcome,
        reason: String,
        maximum_attempts: u16,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompiledWorkflowSemanticConnection {
    pub(super) source: usize,
    pub(super) target: usize,
    pub(super) kind: Arc<CompiledWorkflowConnectionKind>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompiledWorkflowSemanticPlan {
    pub(super) start: usize,
    pub(super) nodes: Box<[Arc<CompiledWorkflowNodeMeaning>]>,
    pub(super) connections: Box<[CompiledWorkflowSemanticConnection]>,
    pub(super) global_actions:
        Arc<[worth_query_installation::facade::WorthQueryWorkflowGlobalActionDependency]>,
    pub(super) dispatch: Arc<CompiledWorkflowDispatch>,
    pub(super) retained_bytes: usize,
}

impl CompiledWorkflowNode {
    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub fn path(&self) -> &str {
        &self.meaning.path
    }

    pub fn kind(&self) -> &CompiledWorkflowNodeKind {
        &self.meaning.kind
    }
}

impl CompiledWorkflowDefinition {
    pub fn lineage(&self) -> EntityId {
        self.publication.lineage
    }

    pub fn definition(&self) -> EntityId {
        self.publication.definition
    }

    pub const fn content_identity(
        &self,
    ) -> &ApplicationWorkflowDefinitionContentIdentity {
        &self.content_identity
    }

    pub const fn program_revision(
        &self,
    ) -> &ApplicationProgramRevision {
        &self.program_revision
    }

    pub fn start_path(&self) -> &str {
        self.start().path()
    }

    pub fn node_count(&self) -> usize {
        self.publication.nodes.len()
    }

    pub fn start(&self) -> &CompiledWorkflowNode {
        &self.publication.nodes[self.resume.unwrap_or(self.publication.start)]
    }

    pub fn node(
        &self,
        entity: EntityId,
    ) -> Option<&CompiledWorkflowNode> {
        self.node_ordinal(entity)
            .map(|ordinal| &self.publication.nodes[ordinal])
    }

    pub fn nodes_with_path(
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

    pub fn control_successors(
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

    pub fn retry_successors(
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

    pub fn required_assessments(
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

    pub fn assessment_subject_sources(
        &self,
        assessment: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.data_sources(assessment, ApplicationWorkflowDataFlow::AssessmentSubject)
    }

    pub fn approval_authority_targets(
        &self,
        approval: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.data_targets(approval, ApplicationWorkflowDataFlow::ApprovalAuthority)
    }

    pub fn approval_authority_sources(
        &self,
        operation: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.data_sources(operation, ApplicationWorkflowDataFlow::ApprovalAuthority)
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

    pub fn approval_proposal_sources(
        &self,
        approval: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.data_sources(approval, ApplicationWorkflowDataFlow::ProposalSubject)
    }

    pub fn approval_evidence_sources(
        &self,
        approval: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.data_sources(approval, ApplicationWorkflowDataFlow::JoinedEvidence)
    }

    pub fn operation_input_sources(
        &self,
        operation: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.data_sources(operation, ApplicationWorkflowDataFlow::OperationInput)
    }

    pub fn captured_operation_input_sources(
        &self,
        operation: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.data_sources(
            operation,
            ApplicationWorkflowDataFlow::CapturedOperationInput,
        )
    }

    pub fn captured_input_targets(
        &self,
        source: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.outgoing(source).filter_map(move |edge| {
            match self.publication.connections[edge.connection].kind.as_ref() {
                CompiledWorkflowConnectionKind::Data(
                    ApplicationWorkflowDataFlow::CapturedOperationInput,
                ) => Some(&self.publication.nodes[edge.peer]),
                _ => None,
            }
        })
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

impl CompiledWorkflowConnection {
    pub fn source(&self) -> EntityId {
        self.source
    }
    pub fn target(&self) -> EntityId {
        self.target
    }
    pub fn kind(
        &self,
    ) -> &CompiledWorkflowConnectionKind {
        &self.kind
    }
}
pub mod court_doors_for_test;
