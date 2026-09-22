use crate::application_program::workflow::ApplicationWorkflowNodeIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowComponentResource {
    Ports,
    Nodes,
    Connections,
    ExpandedNodes,
    ExpandedConnections,
    ComponentOccurrences,
    ComponentDepth,
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
            Self::ComponentDepth => "component depth",
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
    UnknownSequenceNode(ApplicationWorkflowNodeIdentity),
    NonOperationSequenceNode(ApplicationWorkflowNodeIdentity),
    DuplicateSequenceNode(ApplicationWorkflowNodeIdentity),
    InvalidSequenceLength,
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
            Self::UnknownSequenceNode(identity) => {
                write!(
                    formatter,
                    "workflow sequence names absent node: {}",
                    identity.as_str()
                )
            }
            Self::NonOperationSequenceNode(identity) => {
                write!(
                    formatter,
                    "workflow sequence node is not an operation: {}",
                    identity.as_str()
                )
            }
            Self::DuplicateSequenceNode(identity) => {
                write!(
                    formatter,
                    "repeated workflow sequence node: {}",
                    identity.as_str()
                )
            }
            Self::InvalidSequenceLength => {
                formatter.write_str("workflow sequence needs at least two operations")
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
