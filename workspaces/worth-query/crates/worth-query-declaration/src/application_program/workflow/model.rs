use std::marker::PhantomData;

use super::{
    ApplicationWorkflowApprovalRef, ApplicationWorkflowAssessmentRef,
    ApplicationWorkflowComponentExpansion, ApplicationWorkflowConditionRef,
    ApplicationWorkflowDefinitionContentIdentity, ApplicationWorkflowDefinitionIdentity,
    ApplicationWorkflowNodeIdentity, ApplicationWorkflowOperationRef, ApplicationWorkflowSpec,
    ApplicationWorkflowValidationDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowComponentLimits {
    maximum_occurrences: u16,
    maximum_depth: u8,
    maximum_node_provenance: u32,
    maximum_connection_provenance: u32,
    maximum_port_provenance: u32,
}

impl ApplicationWorkflowComponentLimits {
    pub const fn new(
        maximum_occurrences: u16,
        maximum_depth: u8,
        maximum_node_provenance: u32,
        maximum_connection_provenance: u32,
        maximum_port_provenance: u32,
    ) -> Option<Self> {
        if maximum_occurrences == 0
            || maximum_depth == 0
            || maximum_node_provenance == 0
            || maximum_connection_provenance == 0
            || maximum_port_provenance == 0
        {
            None
        } else {
            Some(Self {
                maximum_occurrences,
                maximum_depth,
                maximum_node_provenance,
                maximum_connection_provenance,
                maximum_port_provenance,
            })
        }
    }

    pub const fn maximum_occurrences(self) -> u16 {
        self.maximum_occurrences
    }

    pub const fn maximum_depth(self) -> u8 {
        self.maximum_depth
    }

    pub const fn maximum_node_provenance(self) -> u32 {
        self.maximum_node_provenance
    }

    pub const fn maximum_connection_provenance(self) -> u32 {
        self.maximum_connection_provenance
    }

    pub const fn maximum_port_provenance(self) -> u32 {
        self.maximum_port_provenance
    }

    pub const fn fits_within(self, ceiling: Self) -> bool {
        self.maximum_occurrences <= ceiling.maximum_occurrences
            && self.maximum_depth <= ceiling.maximum_depth
            && self.maximum_node_provenance <= ceiling.maximum_node_provenance
            && self.maximum_connection_provenance <= ceiling.maximum_connection_provenance
            && self.maximum_port_provenance <= ceiling.maximum_port_provenance
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowDefinitionLimits {
    maximum_nodes: u16,
    maximum_connections: u16,
    maximum_effects: u16,
    component_limits: ApplicationWorkflowComponentLimits,
    maximum_canonical_bytes: u32,
}

impl ApplicationWorkflowDefinitionLimits {
    pub const fn new(
        maximum_nodes: u16,
        maximum_connections: u16,
        maximum_effects: u16,
        component_limits: ApplicationWorkflowComponentLimits,
        maximum_canonical_bytes: u32,
    ) -> Option<Self> {
        if maximum_nodes == 0
            || maximum_connections == 0
            || maximum_effects == 0
            || maximum_canonical_bytes == 0
        {
            None
        } else {
            Some(Self {
                maximum_nodes,
                maximum_connections,
                maximum_effects,
                component_limits,
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

    pub const fn component_limits(self) -> ApplicationWorkflowComponentLimits {
        self.component_limits
    }

    pub const fn maximum_component_depth(self) -> u8 {
        self.component_limits.maximum_depth()
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
    Condition(ApplicationWorkflowConditionRef),
    Approval(ApplicationWorkflowApprovalRef),
    EvidenceJoin(ApplicationWorkflowEvidenceJoinPolicy),
    Terminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowEvidenceJoinPolicy {
    AllRequiredPassing,
    AllRequiredCompleted,
}

impl ApplicationWorkflowEvidenceJoinPolicy {
    pub const fn identity(self) -> &'static str {
        match self {
            Self::AllRequiredPassing => "all-required-passing",
            Self::AllRequiredCompleted => "all-required-completed",
        }
    }

    pub const fn from_identity(identity: &str) -> Option<Self> {
        match identity.as_bytes() {
            b"all-required-passing" => Some(Self::AllRequiredPassing),
            b"all-required-completed" => Some(Self::AllRequiredCompleted),
            _ => None,
        }
    }
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
    EvidenceSatisfied,
    EvidenceFailed,
    ConditionSatisfied,
    ConditionUnsatisfied,
    RetryExhausted,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationWorkflowRetry {
    trigger: ApplicationWorkflowControlOutcome,
    reason: String,
    maximum_attempts: u16,
}

impl ApplicationWorkflowRetry {
    pub fn new(
        trigger: ApplicationWorkflowControlOutcome,
        reason: impl Into<String>,
        maximum_attempts: u16,
    ) -> Option<Self> {
        let reason = reason.into();
        (maximum_attempts > 0 && !reason.trim().is_empty()).then_some(Self {
            trigger,
            reason,
            maximum_attempts,
        })
    }

    pub const fn trigger(&self) -> ApplicationWorkflowControlOutcome {
        self.trigger
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
    pub const fn maximum_attempts(&self) -> u16 {
        self.maximum_attempts
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationWorkflowDataFlow {
    ProposalSubject,
    AssessmentSubject,
    ConditionSubject,
    AssessmentEvidence,
    JoinedEvidence,
    ApprovalAuthority,
    OperationInput,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationWorkflowConnectionKind {
    Control(ApplicationWorkflowControlOutcome),
    Data(ApplicationWorkflowDataFlow),
    Retry(ApplicationWorkflowRetry),
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

    pub fn kind(&self) -> ApplicationWorkflowConnectionKind {
        self.kind.clone()
    }

    pub(super) const fn kind_ref(&self) -> &ApplicationWorkflowConnectionKind {
        &self.kind
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
    pub(super) component_expansions: Vec<ApplicationWorkflowComponentExpansion>,
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
    pub(super) component_expansions: Box<[ApplicationWorkflowComponentExpansion]>,
    pub(super) validation_work: super::ApplicationWorkflowValidationWork,
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

    pub fn component_expansions(&self) -> &[ApplicationWorkflowComponentExpansion] {
        &self.component_expansions
    }

    pub const fn validation_work(&self) -> super::ApplicationWorkflowValidationWork {
        self.validation_work
    }
}
