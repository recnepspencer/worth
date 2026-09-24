use super::{
    ApplicationWorkflowComponentIdentity, ApplicationWorkflowConnectionKind,
    ApplicationWorkflowNodeIdentity,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationWorkflowComponentPortDirection {
    Input,
    Output,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowExpandedNodeProvenance {
    authored: ApplicationWorkflowNodeIdentity,
    expanded: ApplicationWorkflowNodeIdentity,
}

impl ApplicationWorkflowExpandedNodeProvenance {
    pub(super) fn new(
        authored: ApplicationWorkflowNodeIdentity,
        expanded: ApplicationWorkflowNodeIdentity,
    ) -> Self {
        Self { authored, expanded }
    }

    pub fn authored(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.authored
    }

    pub fn expanded(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.expanded
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowExpandedPortProvenance {
    identity: String,
    direction: ApplicationWorkflowComponentPortDirection,
    authored_node: ApplicationWorkflowNodeIdentity,
    expanded_node: ApplicationWorkflowNodeIdentity,
}

impl ApplicationWorkflowExpandedPortProvenance {
    pub(super) fn new(
        identity: String,
        direction: ApplicationWorkflowComponentPortDirection,
        authored_node: ApplicationWorkflowNodeIdentity,
        expanded_node: ApplicationWorkflowNodeIdentity,
    ) -> Self {
        Self {
            identity,
            direction,
            authored_node,
            expanded_node,
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn direction(&self) -> ApplicationWorkflowComponentPortDirection {
        self.direction
    }

    pub fn authored_node(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.authored_node
    }

    pub fn expanded_node(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.expanded_node
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowExpandedConnectionProvenance {
    authored_source: ApplicationWorkflowNodeIdentity,
    authored_target: ApplicationWorkflowNodeIdentity,
    expanded_source: ApplicationWorkflowNodeIdentity,
    expanded_target: ApplicationWorkflowNodeIdentity,
    kind: ApplicationWorkflowConnectionKind,
}

impl ApplicationWorkflowExpandedConnectionProvenance {
    pub(super) fn new(
        authored_source: ApplicationWorkflowNodeIdentity,
        authored_target: ApplicationWorkflowNodeIdentity,
        expanded_source: ApplicationWorkflowNodeIdentity,
        expanded_target: ApplicationWorkflowNodeIdentity,
        kind: ApplicationWorkflowConnectionKind,
    ) -> Self {
        Self {
            authored_source,
            authored_target,
            expanded_source,
            expanded_target,
            kind,
        }
    }

    pub fn authored_source(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.authored_source
    }

    pub fn authored_target(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.authored_target
    }

    pub fn expanded_source(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.expanded_source
    }

    pub fn expanded_target(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.expanded_target
    }

    pub fn kind(&self) -> ApplicationWorkflowConnectionKind {
        self.kind.clone()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowComponentExpansion {
    component: ApplicationWorkflowComponentIdentity,
    occurrence_path: String,
    ports: Box<[ApplicationWorkflowExpandedPortProvenance]>,
    nodes: Box<[ApplicationWorkflowExpandedNodeProvenance]>,
    connections: Box<[ApplicationWorkflowExpandedConnectionProvenance]>,
}

impl ApplicationWorkflowComponentExpansion {
    pub(super) fn new(
        component: ApplicationWorkflowComponentIdentity,
        occurrence_path: String,
        ports: Box<[ApplicationWorkflowExpandedPortProvenance]>,
        nodes: Box<[ApplicationWorkflowExpandedNodeProvenance]>,
        connections: Box<[ApplicationWorkflowExpandedConnectionProvenance]>,
    ) -> Self {
        Self {
            component,
            occurrence_path,
            ports,
            nodes,
            connections,
        }
    }

    pub(super) fn qualified(&self, occurrence: &str) -> Result<Self, String> {
        Ok(Self {
            component: self.component.clone(),
            occurrence_path: format!("{occurrence}/{}", self.occurrence_path),
            ports: self
                .ports
                .iter()
                .map(|port| {
                    Ok(ApplicationWorkflowExpandedPortProvenance::new(
                        port.identity.clone(),
                        port.direction,
                        port.authored_node.clone(),
                        port.expanded_node.prefixed(occurrence)?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?
                .into_boxed_slice(),
            nodes: self
                .nodes
                .iter()
                .map(|node| {
                    Ok(ApplicationWorkflowExpandedNodeProvenance::new(
                        node.authored.clone(),
                        node.expanded.prefixed(occurrence)?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?
                .into_boxed_slice(),
            connections: self
                .connections
                .iter()
                .map(|connection| {
                    Ok(ApplicationWorkflowExpandedConnectionProvenance::new(
                        connection.authored_source.clone(),
                        connection.authored_target.clone(),
                        connection.expanded_source.prefixed(occurrence)?,
                        connection.expanded_target.prefixed(occurrence)?,
                        connection.kind.clone(),
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?
                .into_boxed_slice(),
        })
    }

    pub fn component(&self) -> &ApplicationWorkflowComponentIdentity {
        &self.component
    }

    pub fn occurrence_path(&self) -> &str {
        &self.occurrence_path
    }

    pub fn ports(&self) -> &[ApplicationWorkflowExpandedPortProvenance] {
        &self.ports
    }

    pub fn nodes(&self) -> &[ApplicationWorkflowExpandedNodeProvenance] {
        &self.nodes
    }

    pub fn connections(&self) -> &[ApplicationWorkflowExpandedConnectionProvenance] {
        &self.connections
    }
}
