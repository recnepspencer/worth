use std::marker::PhantomData;

use crate::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_query::ApplicationQueryMarkerIdentity,
    application_schema::ApplicationOperationMarkerIdentity,
};

use super::{
    ApplicationWorkflowApprovalRef, ApplicationWorkflowAssessmentRef,
    ApplicationWorkflowConditionRef, ApplicationWorkflowConnection,
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionLimits,
    ApplicationWorkflowEvidenceJoinPolicy, ApplicationWorkflowNode,
    ApplicationWorkflowNodeIdentity, ApplicationWorkflowNodeKind, ApplicationWorkflowOperationRef,
    ApplicationWorkflowSpec, AuthoredWorkflowDefinition,
};

mod command;
mod component;
mod connections;
pub use command::{ApplicationWorkflowAuthoringCommand, ApplicationWorkflowCommandAdapter};
pub use component::{
    ApplicationWorkflowComponentBuilder, ApplicationWorkflowComponentInputBinding,
    ApplicationWorkflowComponentInputPort, ApplicationWorkflowComponentNodeRef,
    ApplicationWorkflowComponentOutputBinding, ApplicationWorkflowComponentOutputPort,
    AuthoredWorkflowComponent, ExpandedWorkflowComponent, ExpandedWorkflowComponentInComponent,
};

pub enum ApplicationWorkflowOperationNode {}
pub enum ApplicationWorkflowAssessmentNode {}
pub enum ApplicationWorkflowConditionNode {}
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

    pub(super) fn qualified(
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

mod binding_seal {
    pub trait Sealed {}
}

#[allow(private_bounds)]
pub trait ApplicationWorkflowInputBinding<Kind>: binding_seal::Sealed {
    #[doc(hidden)]
    fn workflow_node_identity(&self) -> &ApplicationWorkflowNodeIdentity;
}

#[allow(private_bounds)]
pub trait ApplicationWorkflowOutputBinding<Kind>: binding_seal::Sealed {
    #[doc(hidden)]
    fn workflow_node_identity(&self) -> &ApplicationWorkflowNodeIdentity;
}

impl<Kind> binding_seal::Sealed for ApplicationWorkflowNodeRef<Kind> {}

impl<Kind> ApplicationWorkflowInputBinding<Kind> for ApplicationWorkflowNodeRef<Kind> {
    fn workflow_node_identity(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.identity
    }
}

impl<Kind> ApplicationWorkflowOutputBinding<Kind> for ApplicationWorkflowNodeRef<Kind> {
    fn workflow_node_identity(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowComponentResource {
    Ports,
    Nodes,
    Connections,
    ExpandedNodes,
    ExpandedConnections,
    ComponentOccurrences,
    NodeProvenance,
    ConnectionProvenance,
    PortProvenance,
}

impl std::fmt::Display for ApplicationWorkflowComponentResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Ports => "ports",
            Self::Nodes => "nodes",
            Self::Connections => "connections",
            Self::ExpandedNodes => "expanded nodes",
            Self::ExpandedConnections => "expanded connections",
            Self::ComponentOccurrences => "component occurrences",
            Self::NodeProvenance => "node provenance",
            Self::ConnectionProvenance => "connection provenance",
            Self::PortProvenance => "port provenance",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowAuthoringDenial {
    InvalidIdentity(String),
    DuplicateNode(ApplicationWorkflowNodeIdentity),
    UnknownComponentNode(ApplicationWorkflowNodeIdentity),
    DuplicateComponentOccurrence(String),
    DuplicateComponentPort(String),
    ForeignComponentNode,
    ForeignComponentPort,
    ComponentResourceLimitExceeded {
        resource: ApplicationWorkflowComponentResource,
        maximum: u32,
    },
    UnexportedComponentPort(String),
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
            Self::DuplicateComponentOccurrence(occurrence) => {
                write!(
                    formatter,
                    "duplicate workflow component occurrence: {occurrence}"
                )
            }
            Self::DuplicateComponentPort(identity) => {
                write!(formatter, "duplicate workflow component port: {identity}")
            }
            Self::ForeignComponentNode => {
                formatter.write_str("workflow component node belongs to another component")
            }
            Self::ForeignComponentPort => {
                formatter.write_str("workflow component port belongs to another component")
            }
            Self::ComponentResourceLimitExceeded { resource, maximum } => write!(
                formatter,
                "workflow component {resource} exceeds its bounded maximum of {maximum}"
            ),
            Self::UnexportedComponentPort(identity) => {
                write!(
                    formatter,
                    "workflow component port is not exported: {identity}"
                )
            }
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
    component_expansions: Vec<super::ApplicationWorkflowComponentExpansion>,
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
            component_expansions: Vec::new(),
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

    pub fn condition<Query>(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowConditionNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
        Query::ResultBinding:
            crate::application_schema::ApplicationStructuredValueBinding<Value = bool>,
    {
        self.push_node(
            identity,
            ApplicationWorkflowNodeKind::Condition(ApplicationWorkflowConditionRef::declared::<
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
        policy: ApplicationWorkflowEvidenceJoinPolicy,
    ) -> Result<
        ApplicationWorkflowNodeRef<ApplicationWorkflowEvidenceJoinNode>,
        ApplicationWorkflowAuthoringDenial,
    > {
        self.push_node(identity, ApplicationWorkflowNodeKind::EvidenceJoin(policy))
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

    pub fn expand_component(
        &mut self,
        occurrence: &str,
        component: &AuthoredWorkflowComponent<Spec>,
    ) -> Result<ExpandedWorkflowComponent, ApplicationWorkflowAuthoringDenial> {
        component::expand(self, occurrence, component)
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
            component_expansions: self.component_expansions,
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
}
