use std::marker::PhantomData;
use std::sync::Arc;

use crate::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_query::ApplicationQueryMarkerIdentity,
    application_schema::ApplicationOperationMarkerIdentity,
};

use super::{
    ApplicationWorkflowApprovalNode, ApplicationWorkflowAssessmentNode,
    ApplicationWorkflowAuthoringDenial, ApplicationWorkflowComponentResource,
    ApplicationWorkflowConditionNode, ApplicationWorkflowDefinitionBuilder,
    ApplicationWorkflowEvidenceJoinNode, ApplicationWorkflowOperationNode,
    ApplicationWorkflowTerminalNode,
};
use crate::application_program::workflow::{
    ApplicationWorkflowComponentIdentity, ApplicationWorkflowDefinitionLimits,
    ApplicationWorkflowEvidenceJoinPolicy, ApplicationWorkflowNodeIdentity,
    ApplicationWorkflowSpec, ApplicationWorkflowSubjectSelector,
};

mod connections;
mod expansion;
mod model;

pub(super) use expansion::expand;
pub use model::{
    ApplicationWorkflowComponentInputBinding, ApplicationWorkflowComponentInputPort,
    ApplicationWorkflowComponentNodeRef, ApplicationWorkflowComponentOutputBinding,
    ApplicationWorkflowComponentOutputPort, AuthoredWorkflowComponent, ExpandedWorkflowComponent,
    ExpandedWorkflowComponentInComponent,
};
use model::{ComponentAuthoringOwner, ComponentPortDescriptor, PortDirection};

const MAXIMUM_COMPONENT_NODES: u16 = 4_096;
const MAXIMUM_COMPONENT_CONNECTIONS: u16 = 8_192;
const MAXIMUM_COMPONENT_PORTS: u16 = 4_096;
const MAXIMUM_COMPONENT_DEPTH: u8 = 32;
const MAXIMUM_COMPONENT_CANONICAL_BYTES: u32 = 1_048_576;

pub struct ApplicationWorkflowComponentBuilder<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    identity: ApplicationWorkflowComponentIdentity,
    graph: ApplicationWorkflowDefinitionBuilder<Spec>,
    ports: Vec<ComponentPortDescriptor>,
    owner: Arc<ComponentAuthoringOwner>,
}

impl<Spec> ApplicationWorkflowComponentBuilder<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    pub fn new(identity: impl Into<String>) -> Result<Self, ApplicationWorkflowAuthoringDenial> {
        let identity = ApplicationWorkflowComponentIdentity::new(identity)
            .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?;
        let graph = ApplicationWorkflowDefinitionBuilder::new(
            identity.as_str(),
            ApplicationWorkflowDefinitionLimits::new(
                MAXIMUM_COMPONENT_NODES,
                MAXIMUM_COMPONENT_CONNECTIONS,
                MAXIMUM_COMPONENT_NODES,
                MAXIMUM_COMPONENT_DEPTH,
                MAXIMUM_COMPONENT_CANONICAL_BYTES,
            )
            .expect("component construction bounds are nonzero"),
        )?;
        Ok(Self {
            identity,
            graph,
            ports: Vec::new(),
            owner: Arc::new(ComponentAuthoringOwner),
        })
    }

    pub fn operation<Operation>(
        &mut self,
        identity: impl Into<String>,
        requires_workflow_authority: bool,
    ) -> Result<
        ApplicationWorkflowComponentNodeRef<ApplicationWorkflowOperationNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Spec::Schema> + 'static,
    {
        self.require_node_capacity()?;
        self.graph
            .operation::<Operation>(identity, requires_workflow_authority)
            .map(|inner| ApplicationWorkflowComponentNodeRef {
                inner,
                owner: Arc::clone(&self.owner),
            })
    }

    pub fn assessment<Query>(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowComponentNodeRef<ApplicationWorkflowAssessmentNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
    {
        self.assessment_for::<Query>(identity, ApplicationWorkflowSubjectSelector::Resource)
    }

    pub fn assessment_for<Query>(
        &mut self,
        identity: impl Into<String>,
        subject: ApplicationWorkflowSubjectSelector,
    ) -> Result<
        ApplicationWorkflowComponentNodeRef<ApplicationWorkflowAssessmentNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
    {
        self.require_node_capacity()?;
        self.graph
            .assessment_for::<Query>(identity, subject)
            .map(|inner| ApplicationWorkflowComponentNodeRef {
                inner,
                owner: Arc::clone(&self.owner),
            })
    }

    pub fn condition<Query>(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowComponentNodeRef<ApplicationWorkflowConditionNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
        Query::ResultBinding:
            crate::application_schema::ApplicationStructuredValueBinding<Value = bool>,
    {
        self.require_node_capacity()?;
        self.graph
            .condition::<Query>(identity)
            .map(|inner| ApplicationWorkflowComponentNodeRef {
                inner,
                owner: Arc::clone(&self.owner),
            })
    }

    pub fn approval<Capability>(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowComponentNodeRef<ApplicationWorkflowApprovalNode>,
        ApplicationWorkflowAuthoringDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Spec::Schema> + 'static,
    {
        self.require_node_capacity()?;
        self.graph.approval::<Capability>(identity).map(|inner| {
            ApplicationWorkflowComponentNodeRef {
                inner,
                owner: Arc::clone(&self.owner),
            }
        })
    }

    pub fn evidence_join(
        &mut self,
        identity: impl Into<String>,
        policy: ApplicationWorkflowEvidenceJoinPolicy,
    ) -> Result<
        ApplicationWorkflowComponentNodeRef<ApplicationWorkflowEvidenceJoinNode>,
        ApplicationWorkflowAuthoringDenial,
    > {
        self.require_node_capacity()?;
        self.graph.evidence_join(identity, policy).map(|inner| {
            ApplicationWorkflowComponentNodeRef {
                inner,
                owner: Arc::clone(&self.owner),
            }
        })
    }

    pub fn terminal(
        &mut self,
        identity: impl Into<String>,
    ) -> Result<
        ApplicationWorkflowComponentNodeRef<ApplicationWorkflowTerminalNode>,
        ApplicationWorkflowAuthoringDenial,
    > {
        self.require_node_capacity()?;
        self.graph
            .terminal(identity)
            .map(|inner| ApplicationWorkflowComponentNodeRef {
                inner,
                owner: Arc::clone(&self.owner),
            })
    }

    pub fn expand_component(
        &mut self,
        occurrence: &str,
        component: &AuthoredWorkflowComponent<Spec>,
    ) -> Result<ExpandedWorkflowComponentInComponent, ApplicationWorkflowAuthoringDenial> {
        let expanded = self.graph.expand_component(occurrence, component)?;
        Ok(ExpandedWorkflowComponentInComponent {
            component: expanded.component,
            occurrence: expanded.occurrence,
            ports: expanded.ports,
            component_owner: expanded.owner,
            owner: Arc::clone(&self.owner),
        })
    }

    pub fn input_port<Kind>(
        &mut self,
        identity: impl Into<String>,
        node: &ApplicationWorkflowComponentNodeRef<Kind>,
    ) -> Result<ApplicationWorkflowComponentInputPort<Kind>, ApplicationWorkflowAuthoringDenial>
    {
        self.require_owned(node)?;
        let identity = self.export(identity, node.identity(), PortDirection::Input)?;
        Ok(ApplicationWorkflowComponentInputPort {
            component: self.identity.clone(),
            identity,
            node: node.clone(),
        })
    }

    pub fn output_port<Kind>(
        &mut self,
        identity: impl Into<String>,
        node: &ApplicationWorkflowComponentNodeRef<Kind>,
    ) -> Result<ApplicationWorkflowComponentOutputPort<Kind>, ApplicationWorkflowAuthoringDenial>
    {
        self.require_owned(node)?;
        let identity = self.export(identity, node.identity(), PortDirection::Output)?;
        Ok(ApplicationWorkflowComponentOutputPort {
            component: self.identity.clone(),
            identity,
            node: node.clone(),
        })
    }

    pub fn finish(
        self,
    ) -> Result<AuthoredWorkflowComponent<Spec>, ApplicationWorkflowAuthoringDenial> {
        let known = self
            .graph
            .nodes
            .iter()
            .map(|node| node.identity().clone())
            .collect::<std::collections::BTreeSet<_>>();
        for connection in &self.graph.connections {
            if !known.contains(connection.source()) {
                return Err(ApplicationWorkflowAuthoringDenial::UnknownComponentNode(
                    connection.source().clone(),
                ));
            }
            if !known.contains(connection.target()) {
                return Err(ApplicationWorkflowAuthoringDenial::UnknownComponentNode(
                    connection.target().clone(),
                ));
            }
        }
        for port in &self.ports {
            if !known.contains(&port.node) {
                return Err(ApplicationWorkflowAuthoringDenial::UnknownComponentNode(
                    port.node.clone(),
                ));
            }
        }
        Ok(AuthoredWorkflowComponent {
            identity: self.identity,
            nodes: self.graph.nodes,
            connections: self.graph.connections,
            ports: self.ports,
            component_expansions: self.graph.component_expansions,
            owner: self.owner,
            marker: PhantomData,
        })
    }

    fn export(
        &mut self,
        identity: impl Into<String>,
        node: &ApplicationWorkflowNodeIdentity,
        direction: PortDirection,
    ) -> Result<String, ApplicationWorkflowAuthoringDenial> {
        if self.ports.len() >= usize::from(MAXIMUM_COMPONENT_PORTS) {
            return Err(
                ApplicationWorkflowAuthoringDenial::ComponentResourceLimitExceeded {
                    resource: ApplicationWorkflowComponentResource::Ports,
                    maximum: u32::from(MAXIMUM_COMPONENT_PORTS),
                },
            );
        }
        let identity = identity.into();
        super::super::identity::require_identity(&identity)
            .map_err(ApplicationWorkflowAuthoringDenial::InvalidIdentity)?;
        if self.ports.iter().any(|port| port.identity == identity) {
            return Err(ApplicationWorkflowAuthoringDenial::DuplicateComponentPort(
                identity,
            ));
        }
        if !self
            .graph
            .nodes
            .iter()
            .any(|candidate| candidate.identity() == node)
        {
            return Err(ApplicationWorkflowAuthoringDenial::UnknownComponentNode(
                node.clone(),
            ));
        }
        self.ports.push(ComponentPortDescriptor {
            identity: identity.clone(),
            node: node.clone(),
            direction,
        });
        Ok(identity)
    }

    fn require_owned<Kind>(
        &self,
        node: &ApplicationWorkflowComponentNodeRef<Kind>,
    ) -> Result<(), ApplicationWorkflowAuthoringDenial> {
        Arc::ptr_eq(&self.owner, &node.owner)
            .then_some(())
            .ok_or(ApplicationWorkflowAuthoringDenial::ForeignComponentNode)
    }

    fn require_node_capacity(&self) -> Result<(), ApplicationWorkflowAuthoringDenial> {
        if self.graph.nodes.len() >= usize::from(self.graph.limits.maximum_nodes()) {
            Err(
                ApplicationWorkflowAuthoringDenial::ComponentResourceLimitExceeded {
                    resource: ApplicationWorkflowComponentResource::Nodes,
                    maximum: u32::from(self.graph.limits.maximum_nodes()),
                },
            )
        } else {
            Ok(())
        }
    }
}
