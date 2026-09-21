use std::marker::PhantomData;

use super::{
    ApplicationWorkflowApprovalRef, ApplicationWorkflowAssessmentRef,
    ApplicationWorkflowDefinitionContentIdentity, ApplicationWorkflowDefinitionIdentity,
    ApplicationWorkflowNodeIdentity, ApplicationWorkflowOperationRef, ApplicationWorkflowSpec,
    ApplicationWorkflowValidationDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowDefinitionLimits {
    maximum_nodes: u16,
    maximum_connections: u16,
    maximum_effects: u16,
    maximum_component_depth: u8,
    maximum_canonical_bytes: u32,
}

impl ApplicationWorkflowDefinitionLimits {
    pub const fn new(
        maximum_nodes: u16,
        maximum_connections: u16,
        maximum_effects: u16,
        maximum_component_depth: u8,
        maximum_canonical_bytes: u32,
    ) -> Option<Self> {
        if maximum_nodes == 0
            || maximum_connections == 0
            || maximum_effects == 0
            || maximum_component_depth == 0
            || maximum_canonical_bytes == 0
        {
            None
        } else {
            Some(Self {
                maximum_nodes,
                maximum_connections,
                maximum_effects,
                maximum_component_depth,
                maximum_canonical_bytes,
            })
        }
    }

    pub const fn maximum_nodes(self) -> u16 {
        self.maximum_nodes
    }

    pub const fn maximum_connections(self) -> u16 {
        self.maximum_connections
    }

    pub const fn maximum_effects(self) -> u16 {
        self.maximum_effects
    }

    pub const fn maximum_component_depth(self) -> u8 {
        self.maximum_component_depth
    }

    pub const fn maximum_canonical_bytes(self) -> u32 {
        self.maximum_canonical_bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowNodeKind {
    Operation {
        operation: ApplicationWorkflowOperationRef,
        requires_workflow_authority: bool,
    },
    Assessment(ApplicationWorkflowAssessmentRef),
    Approval(ApplicationWorkflowApprovalRef),
    EvidenceJoin,
    Terminal,
}

impl ApplicationWorkflowNodeKind {
    pub const fn is_effect(&self) -> bool {
        matches!(self, Self::Operation { .. })
    }

    pub const fn requires_workflow_authority(&self) -> bool {
        matches!(
            self,
            Self::Operation {
                requires_workflow_authority: true,
                ..
            }
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowNode {
    identity: ApplicationWorkflowNodeIdentity,
    kind: ApplicationWorkflowNodeKind,
}

impl ApplicationWorkflowNode {
    pub(super) fn new(
        identity: ApplicationWorkflowNodeIdentity,
        kind: ApplicationWorkflowNodeKind,
    ) -> Self {
        Self { identity, kind }
    }

    pub fn identity(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.identity
    }

    pub fn kind(&self) -> &ApplicationWorkflowNodeKind {
        &self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationWorkflowControlOutcome {
    Completed,
    Approved,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationWorkflowDataFlow {
    ProposalSubject,
    AssessmentSubject,
    AssessmentEvidence,
    JoinedEvidence,
    ApprovalAuthority,
    OperationInput,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationWorkflowConnectionKind {
    Control(ApplicationWorkflowControlOutcome),
    Data(ApplicationWorkflowDataFlow),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowConnection {
    source: ApplicationWorkflowNodeIdentity,
    target: ApplicationWorkflowNodeIdentity,
    kind: ApplicationWorkflowConnectionKind,
}

impl ApplicationWorkflowConnection {
    pub(super) fn new(
        source: ApplicationWorkflowNodeIdentity,
        target: ApplicationWorkflowNodeIdentity,
        kind: ApplicationWorkflowConnectionKind,
    ) -> Self {
        Self {
            source,
            target,
            kind,
        }
    }

    pub fn source(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.source
    }

    pub fn target(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.target
    }

    pub const fn kind(&self) -> ApplicationWorkflowConnectionKind {
        self.kind
    }
}

pub struct AuthoredWorkflowDefinition<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    pub(super) identity: ApplicationWorkflowDefinitionIdentity,
    pub(super) limits: ApplicationWorkflowDefinitionLimits,
    pub(super) start: Option<ApplicationWorkflowNodeIdentity>,
    pub(super) nodes: Vec<ApplicationWorkflowNode>,
    pub(super) connections: Vec<ApplicationWorkflowConnection>,
    pub(super) marker: PhantomData<fn() -> Spec>,
}

impl<Spec> AuthoredWorkflowDefinition<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    pub fn validate(
        self,
    ) -> Result<ValidatedWorkflowDefinition<Spec>, ApplicationWorkflowValidationDenial> {
        super::validation::validate(self)
    }
}

/// Pure, canonical, validated workflow meaning.
///
/// It carries no branch currentness, installed support, publication status, or
/// authority to start or execute an instance.
pub struct ValidatedWorkflowDefinition<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    pub(super) identity: ApplicationWorkflowDefinitionIdentity,
    pub(super) content_identity: ApplicationWorkflowDefinitionContentIdentity,
    pub(super) limits: ApplicationWorkflowDefinitionLimits,
    pub(super) start: ApplicationWorkflowNodeIdentity,
    pub(super) nodes: Box<[ApplicationWorkflowNode]>,
    pub(super) connections: Box<[ApplicationWorkflowConnection]>,
    pub(super) marker: PhantomData<fn() -> Spec>,
}

impl<Spec> ValidatedWorkflowDefinition<Spec>
where
    Spec: ApplicationWorkflowSpec,
{
    pub fn identity(&self) -> &ApplicationWorkflowDefinitionIdentity {
        &self.identity
    }

    pub fn content_identity(&self) -> &ApplicationWorkflowDefinitionContentIdentity {
        &self.content_identity
    }

    pub const fn limits(&self) -> ApplicationWorkflowDefinitionLimits {
        self.limits
    }

    pub fn start(&self) -> &ApplicationWorkflowNodeIdentity {
        &self.start
    }

    pub fn nodes(&self) -> &[ApplicationWorkflowNode] {
        &self.nodes
    }

    pub fn connections(&self) -> &[ApplicationWorkflowConnection] {
        &self.connections
    }
}
