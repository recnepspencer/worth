use std::marker::PhantomData;
use std::sync::Arc;

use super::super::{
    binding_seal, ApplicationWorkflowAuthoringDenial, ApplicationWorkflowInputBinding,
    ApplicationWorkflowNodeRef, ApplicationWorkflowOutputBinding,
};
use crate::application_program::workflow::{
    ApplicationWorkflowComponentExpansion, ApplicationWorkflowComponentIdentity,
    ApplicationWorkflowComponentPortDirection, ApplicationWorkflowConnection,
    ApplicationWorkflowNode, ApplicationWorkflowNodeIdentity, ApplicationWorkflowSpec,
};

pub(super) type PortDirection = ApplicationWorkflowComponentPortDirection;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ComponentPortDescriptor {
    pub(super) identity: String,
    pub(super) node: ApplicationWorkflowNodeIdentity,
    pub(super) direction: PortDirection,
}

/// A node handle scoped to component authoring.
///
/// It is accepted by component connections:
/// ```
/// use worth_query_declaration::facade::application_program::{
///     ApplicationWorkflowComponentBuilder, ApplicationWorkflowComponentNodeRef,
///     ApplicationWorkflowOperationNode, ApplicationWorkflowSpec,
/// };
/// fn connect_inside_component<Spec: ApplicationWorkflowSpec>(
///     builder: &mut ApplicationWorkflowComponentBuilder<Spec>,
///     source: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowOperationNode>,
///     target: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowOperationNode>,
/// ) {
///     builder.operation_input(source, target);
/// }
/// ```
///
/// It cannot bypass a component port and enter definition authoring:
/// ```compile_fail
/// use worth_query_declaration::facade::application_program::{
///     ApplicationWorkflowComponentNodeRef, ApplicationWorkflowDefinitionBuilder,
///     ApplicationWorkflowOperationNode, ApplicationWorkflowSpec,
/// };
/// fn bypass_exported_ports<Spec: ApplicationWorkflowSpec>(
///     builder: &mut ApplicationWorkflowDefinitionBuilder<Spec>,
///     source: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowOperationNode>,
///     target: &ApplicationWorkflowComponentNodeRef<ApplicationWorkflowOperationNode>,
/// ) {
///     builder.operation_input(source, target);
/// }
/// ```
#[derive(Debug)]
pub struct ApplicationWorkflowComponentNodeRef<Kind> {
    pub(super) inner: ApplicationWorkflowNodeRef<Kind>,
    pub(super) owner: Arc<ComponentAuthoringOwner>,
}

#[derive(Debug)]
pub(super) struct ComponentAuthoringOwner;

impl<Kind> PartialEq for ApplicationWorkflowComponentNodeRef<Kind> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.identity() == other.inner.identity() && Arc::ptr_eq(&self.owner, &other.owner)
    }
}

impl<Kind> Eq for ApplicationWorkflowComponentNodeRef<Kind> {}

impl<Kind> Clone for ApplicationWorkflowComponentNodeRef<Kind> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            owner: Arc::clone(&self.owner),
        }
    }
}

impl<Kind> ApplicationWorkflowComponentNodeRef<Kind> {
    pub fn identity(&self) -> &ApplicationWorkflowNodeIdentity {
        self.inner.identity()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowComponentInputPort<Kind> {
    pub(super) component: ApplicationWorkflowComponentIdentity,
    pub(super) identity: String,
    pub(super) node: ApplicationWorkflowComponentNodeRef<Kind>,
}

impl<Kind> Clone for ApplicationWorkflowComponentInputPort<Kind> {
    fn clone(&self) -> Self {
        Self {
            component: self.component.clone(),
            identity: self.identity.clone(),
            node: self.node.clone(),
        }
    }
}

impl<Kind> ApplicationWorkflowComponentInputPort<Kind> {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn component(&self) -> &ApplicationWorkflowComponentIdentity {
        &self.component
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowComponentOutputPort<Kind> {
    pub(super) component: ApplicationWorkflowComponentIdentity,
    pub(super) identity: String,
    pub(super) node: ApplicationWorkflowComponentNodeRef<Kind>,
}

impl<Kind> Clone for ApplicationWorkflowComponentOutputPort<Kind> {
    fn clone(&self) -> Self {
        Self {
            component: self.component.clone(),
            identity: self.identity.clone(),
            node: self.node.clone(),
        }
    }
}

impl<Kind> ApplicationWorkflowComponentOutputPort<Kind> {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn component(&self) -> &ApplicationWorkflowComponentIdentity {
        &self.component
    }
}

pub struct AuthoredWorkflowComponent<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    pub(super) identity: ApplicationWorkflowComponentIdentity,
    pub(super) nodes: Vec<ApplicationWorkflowNode>,
    pub(super) connections: Vec<ApplicationWorkflowConnection>,
    pub(super) ports: Vec<ComponentPortDescriptor>,
    pub(super) component_expansions: Vec<ApplicationWorkflowComponentExpansion>,
    pub(super) owner: Arc<ComponentAuthoringOwner>,
    pub(super) marker: PhantomData<fn() -> Spec>,
}

impl<Spec> AuthoredWorkflowComponent<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    pub fn identity(&self) -> &ApplicationWorkflowComponentIdentity {
        &self.identity
    }
}

pub struct ExpandedWorkflowComponent {
    pub(super) component: ApplicationWorkflowComponentIdentity,
    pub(super) occurrence: String,
    pub(super) ports: Vec<ComponentPortDescriptor>,
    pub(super) owner: Arc<ComponentAuthoringOwner>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowComponentInputBinding<Kind> {
    node: ApplicationWorkflowNodeRef<Kind>,
}

impl<Kind> ApplicationWorkflowComponentInputBinding<Kind> {
    pub fn identity(&self) -> &ApplicationWorkflowNodeIdentity {
        self.node.identity()
    }
}

impl<Kind> binding_seal::Sealed for ApplicationWorkflowComponentInputBinding<Kind> {}

impl<Kind> ApplicationWorkflowInputBinding<Kind>
    for ApplicationWorkflowComponentInputBinding<Kind>
{
    fn workflow_node_identity(&self) -> &ApplicationWorkflowNodeIdentity {
        self.node.identity()
    }
}

/// A qualified component output. It cannot be used as an incoming binding.
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_program::{
///     ApplicationWorkflowComponentOutputBinding, ApplicationWorkflowDefinitionBuilder,
///     ApplicationWorkflowAssessmentNode, ApplicationWorkflowOperationNode,
///     ApplicationWorkflowNodeRef, ApplicationWorkflowSpec,
/// };
/// fn reverse_port_direction<Spec: ApplicationWorkflowSpec>(
///     builder: &mut ApplicationWorkflowDefinitionBuilder<Spec>,
///     source: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
///     output: &ApplicationWorkflowComponentOutputBinding<ApplicationWorkflowAssessmentNode>,
/// ) {
///     builder.proposal_for_assessment(source, output);
/// }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowComponentOutputBinding<Kind> {
    node: ApplicationWorkflowNodeRef<Kind>,
}

impl<Kind> ApplicationWorkflowComponentOutputBinding<Kind> {
    pub fn identity(&self) -> &ApplicationWorkflowNodeIdentity {
        self.node.identity()
    }
}

impl<Kind> binding_seal::Sealed for ApplicationWorkflowComponentOutputBinding<Kind> {}

impl<Kind> ApplicationWorkflowOutputBinding<Kind>
    for ApplicationWorkflowComponentOutputBinding<Kind>
{
    fn workflow_node_identity(&self) -> &ApplicationWorkflowNodeIdentity {
        self.node.identity()
    }
}

pub struct ExpandedWorkflowComponentInComponent {
    pub(super) component: ApplicationWorkflowComponentIdentity,
    pub(super) occurrence: String,
    pub(super) ports: Vec<ComponentPortDescriptor>,
    pub(super) component_owner: Arc<ComponentAuthoringOwner>,
    pub(super) owner: Arc<ComponentAuthoringOwner>,
}

impl ExpandedWorkflowComponent {
    pub fn input<Kind>(
        &self,
        port: &ApplicationWorkflowComponentInputPort<Kind>,
    ) -> Result<ApplicationWorkflowComponentInputBinding<Kind>, ApplicationWorkflowAuthoringDenial>
    {
        bind_port(
            &self.component,
            &self.owner,
            &self.ports,
            &port.component,
            &port.node.owner,
            &port.identity,
            port.node.identity(),
            PortDirection::Input,
        )?;
        Ok(ApplicationWorkflowComponentInputBinding {
            node: port.node.inner.qualified(&self.occurrence)?,
        })
    }

    pub fn output<Kind>(
        &self,
        port: &ApplicationWorkflowComponentOutputPort<Kind>,
    ) -> Result<ApplicationWorkflowComponentOutputBinding<Kind>, ApplicationWorkflowAuthoringDenial>
    {
        bind_port(
            &self.component,
            &self.owner,
            &self.ports,
            &port.component,
            &port.node.owner,
            &port.identity,
            port.node.identity(),
            PortDirection::Output,
        )?;
        Ok(ApplicationWorkflowComponentOutputBinding {
            node: port.node.inner.qualified(&self.occurrence)?,
        })
    }
}

impl ExpandedWorkflowComponentInComponent {
    pub fn input<Kind>(
        &self,
        port: &ApplicationWorkflowComponentInputPort<Kind>,
    ) -> Result<ApplicationWorkflowComponentNodeRef<Kind>, ApplicationWorkflowAuthoringDenial> {
        bind_port(
            &self.component,
            &self.component_owner,
            &self.ports,
            &port.component,
            &port.node.owner,
            &port.identity,
            port.node.identity(),
            PortDirection::Input,
        )?;
        Ok(ApplicationWorkflowComponentNodeRef {
            inner: port.node.inner.qualified(&self.occurrence)?,
            owner: Arc::clone(&self.owner),
        })
    }

    pub fn output<Kind>(
        &self,
        port: &ApplicationWorkflowComponentOutputPort<Kind>,
    ) -> Result<ApplicationWorkflowComponentNodeRef<Kind>, ApplicationWorkflowAuthoringDenial> {
        bind_port(
            &self.component,
            &self.component_owner,
            &self.ports,
            &port.component,
            &port.node.owner,
            &port.identity,
            port.node.identity(),
            PortDirection::Output,
        )?;
        Ok(ApplicationWorkflowComponentNodeRef {
            inner: port.node.inner.qualified(&self.occurrence)?,
            owner: Arc::clone(&self.owner),
        })
    }
}

fn bind_port(
    occurrence_component: &ApplicationWorkflowComponentIdentity,
    occurrence_component_owner: &Arc<ComponentAuthoringOwner>,
    ports: &[ComponentPortDescriptor],
    port_component: &ApplicationWorkflowComponentIdentity,
    port_component_owner: &Arc<ComponentAuthoringOwner>,
    identity: &str,
    node: &ApplicationWorkflowNodeIdentity,
    direction: PortDirection,
) -> Result<(), ApplicationWorkflowAuthoringDenial> {
    if port_component != occurrence_component
        || !Arc::ptr_eq(port_component_owner, occurrence_component_owner)
    {
        return Err(ApplicationWorkflowAuthoringDenial::ForeignComponentPort);
    }
    ports
        .iter()
        .any(|port| port.identity == identity && port.node == *node && port.direction == direction)
        .then_some(())
        .ok_or_else(|| ApplicationWorkflowAuthoringDenial::UnexportedComponentPort(identity.into()))
}
