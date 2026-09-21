use std::marker::PhantomData;

use crate::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_query::ApplicationQueryMarkerIdentity,
    application_schema::ApplicationOperationMarkerIdentity,
};

use super::{
    ApplicationWorkflowApprovalRef, ApplicationWorkflowAssessmentRef,
    ApplicationWorkflowConnection, ApplicationWorkflowConnectionKind,
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDataFlow,
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionLimits,
    ApplicationWorkflowNode, ApplicationWorkflowNodeIdentity, ApplicationWorkflowNodeKind,
    ApplicationWorkflowOperationRef, ApplicationWorkflowSpec, AuthoredWorkflowDefinition,
};

mod command;
pub use command::{ApplicationWorkflowAuthoringCommand, ApplicationWorkflowCommandAdapter};

pub enum ApplicationWorkflowOperationNode {}
pub enum ApplicationWorkflowAssessmentNode {}
pub enum ApplicationWorkflowApprovalNode {}
pub enum ApplicationWorkflowEvidenceJoinNode {}
pub enum ApplicationWorkflowTerminalNode {}

#[derive(Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowNodeRef<Kind> {
    identity: ApplicationWorkflowNodeIdentity,
    marker: PhantomData<fn() -> Kind>,
}

impl<Kind> Clone for ApplicationWorkflowNodeRef<Kind> {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            marker: PhantomData,
        }
    }
}

impl<Kind> ApplicationWorkflowNodeRef<Kind> {
    pub fn identity(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.identity
    }

    pub fn in_component(
        &self,
        occurrence: &str,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial> {
        Ok(Self {
            identity: self
                .identity
                .prefixed(occurrence)
                .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?,
            marker: PhantomData,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowAuthoringDenial {
    InvalidIdentity(String),
    DuplicateNode(ApplicationWorkflowNodeIdentity),
    UnknownComponentNode(ApplicationWorkflowNodeIdentity),
}

impl std::fmt::Display for ApplicationWorkflowAuthoringDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIdentity(identity) => {
                write!(formatter, "invalid workflow identity: {identity}")
            }
            Self::DuplicateNode(identity) => {
                write!(formatter, "duplicate workflow node: {}", identity.as_str())
            }
            Self::UnknownComponentNode(identity) => write!(
                formatter,
                "component connection names absent node: {}",
                identity.as_str()
            ),
        }
    }
}

impl std::error::Error for ApplicationWorkflowAuthoringDenial {}

pub struct ApplicationWorkflowDefinitionBuilder<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    identity: ApplicationWorkflowDefinitionIdentity,
    limits: ApplicationWorkflowDefinitionLimits,
    start: Option<ApplicationWorkflowNodeIdentity>,
    nodes: Vec<ApplicationWorkflowNode>,
    connections: Vec<ApplicationWorkflowConnection>,
    marker: PhantomData<fn() -> Spec>,
}

impl<Spec> ApplicationWorkflowDefinitionBuilder<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    pub fn new(
        identity: impl Into<String>,
        limits: ApplicationWorkflowDefinitionLimits,
    ) -> Result<Self, ApplicationWorkflowAuthoringDenial> {
        Ok(Self {
            identity: ApplicationWorkflowDefinitionIdentity::new(identity)
                .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?,
            limits,
            start: None,
            nodes: Vec::new(),
            connections: Vec::new(),
            marker: PhantomData,
        })
    }

    pub fn operation<Operation>(
        &mut self,
        identity: impl Into<String>,
        requires_workflow_authority: bool,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Spec::Schema> + 'static,
    {
        self.push_node(
            identity,
            ApplicationWorkflowNodeKind::Operation {
                operation: ApplicationWorkflowOperationRef::declared::<Spec, Operation>(),
                requires_workflow_authority,
            },
        )
    }

    pub fn assessment<Query>(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowAssessmentNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
    {
        self.push_node(
            identity,
            ApplicationWorkflowNodeKind::Assessment(ApplicationWorkflowAssessmentRef::declared::<
                Spec,
                Query,
            >()),
        )
    }

    pub fn approval<Capability>(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowApprovalNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Spec::Schema> + 'static,
    {
        self.push_node(
            identity,
            ApplicationWorkflowNodeKind::Approval(ApplicationWorkflowApprovalRef::declared::<
                Spec,
                Capability,
            >()),
        )
    }

    pub fn evidence_join(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowEvidenceJoinNode>,
        ApplicationWorkflowAuthoringDenial,
    > {
        self.push_node(identity, ApplicationWorkflowNodeKind::EvidenceJoin)
    }

    pub fn terminal(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowTerminalNode>,
        ApplicationWorkflowAuthoringDenial,
    > {
        self.push_node(identity, ApplicationWorkflowNodeKind::Terminal)
    }

    pub fn start<Kind>(&mut self, node: &ApplicationWorkflowNodeRef<Kind>) -> &mut Self {
        self.start = Some(node.identity.clone());
        self
    }

    pub fn control<Source, Target>(
        &mut self,
        source: &ApplicationWorkflowNodeRef<Source>,
        outcome: ApplicationWorkflowControlOutcome,
        target: &ApplicationWorkflowNodeRef<Target>,
    ) -> &mut Self {
        self.connect(
            source,
            target,
            ApplicationWorkflowConnectionKind::Control(outcome),
        )
    }

    pub fn proposal_for_assessment(
        &mut self,
        source: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
        target: &ApplicationWorkflowNodeRef<ApplicationWorkflowAssessmentNode>,
    ) -> &mut Self {
        self.connect_data(
            source,
            target,
            ApplicationWorkflowDataFlow::AssessmentSubject,
        )
    }

    pub fn proposal_for_approval(
        &mut self,
        source: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
        target: &ApplicationWorkflowNodeRef<ApplicationWorkflowApprovalNode>,
    ) -> &mut Self {
        self.connect_data(source, target, ApplicationWorkflowDataFlow::ProposalSubject)
    }

    pub fn assessment_evidence(
        &mut self,
        source: &ApplicationWorkflowNodeRef<ApplicationWorkflowAssessmentNode>,
        target: &ApplicationWorkflowNodeRef<ApplicationWorkflowEvidenceJoinNode>,
    ) -> &mut Self {
        self.connect_data(
            source,
            target,
            ApplicationWorkflowDataFlow::AssessmentEvidence,
        )
    }

    pub fn joined_evidence(
        &mut self,
        source: &ApplicationWorkflowNodeRef<ApplicationWorkflowEvidenceJoinNode>,
        target: &ApplicationWorkflowNodeRef<ApplicationWorkflowApprovalNode>,
    ) -> &mut Self {
        self.connect_data(source, target, ApplicationWorkflowDataFlow::JoinedEvidence)
    }

    pub fn approval_authority(
        &mut self,
        source: &ApplicationWorkflowNodeRef<ApplicationWorkflowApprovalNode>,
        target: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
    ) -> &mut Self {
        self.connect_data(
            source,
            target,
            ApplicationWorkflowDataFlow::ApprovalAuthority,
        )
    }

    pub fn operation_input(
        &mut self,
        source: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
        target: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
    ) -> &mut Self {
        self.connect_data(source, target, ApplicationWorkflowDataFlow::OperationInput)
    }

    pub fn expand_component(
        &mut self,
        occurrence: &str,
        component: AuthoredWorkflowDefinition<Spec>,
    ) -> Result<&mut Self, ApplicationWorkflowAuthoringDenial> {
        super::identity::require_identity(occurrence)
            .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?;
        let known = component
            .nodes
            .iter()
            .map(|node| node.identity().clone())
            .collect::<std::collections::BTreeSet<_>>();
        for connection in &component.connections {
            if !known.contains(connection.source()) || !known.contains(connection.target()) {
                return Err(ApplicationWorkflowAuthoringDenial::UnknownComponentNode(
                    connection.source().clone(),
                ));
            }
        }
        for node in component.nodes {
            let identity = node
                .identity()
                .prefixed(occurrence)
                .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?;
            self.push_erased(identity, node.kind().clone())?;
        }
        for connection in component.connections {
            self.connections.push(ApplicationWorkflowConnection::new(
                connection
                    .source()
                    .prefixed(occurrence)
                    .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?,
                connection
                    .target()
                    .prefixed(occurrence)
                    .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?,
                connection.kind(),
            ));
        }
        Ok(self)
    }

    pub fn finish(
        self,
    ) -> Result<AuthoredWorkflowDefinition<Spec>, ApplicationWorkflowAuthoringDenial> {
        Ok(AuthoredWorkflowDefinition {
            identity: self.identity,
            limits: self.limits,
            start: self.start,
            nodes: self.nodes,
            connections: self.connections,
            marker: PhantomData,
        })
    }

    fn push_node<Kind>(
        &mut self,
        identity: impl Into<String>,
        kind: ApplicationWorkflowNodeKind,
    ) -> Result<ApplicationWorkflowNodeRef<Kind>, ApplicationWorkflowAuthoringDenial> {
        let identity = ApplicationWorkflowNodeIdentity::new(identity)
            .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?;
        self.push_erased(identity.clone(), kind)?;
        Ok(ApplicationWorkflowNodeRef {
            identity,
            marker: PhantomData,
        })
    }

    fn push_erased(
        &mut self,
        identity: ApplicationWorkflowNodeIdentity,
        kind: ApplicationWorkflowNodeKind,
    ) -> Result<(), ApplicationWorkflowAuthoringDenial> {
        if self.nodes.iter().any(|node| node.identity() == &identity) {
            return Err(ApplicationWorkflowAuthoringDenial::DuplicateNode(identity));
        }
        self.nodes
            .push(ApplicationWorkflowNode::new(identity, kind));
        Ok(())
    }

    fn connect<Source, Target>(
        &mut self,
        source: &ApplicationWorkflowNodeRef<Source>,
        target: &ApplicationWorkflowNodeRef<Target>,
        kind: ApplicationWorkflowConnectionKind,
    ) -> &mut Self {
        self.connections.push(ApplicationWorkflowConnection::new(
            source.identity.clone(),
            target.identity.clone(),
            kind,
        ));
        self
    }

    fn connect_data<Source, Target>(
        &mut self,
        source: &ApplicationWorkflowNodeRef<Source>,
        target: &ApplicationWorkflowNodeRef<Target>,
        flow: ApplicationWorkflowDataFlow,
    ) -> &mut Self {
        self.connect(
            source,
            target,
            ApplicationWorkflowConnectionKind::Data(flow),
        )
    }
}
