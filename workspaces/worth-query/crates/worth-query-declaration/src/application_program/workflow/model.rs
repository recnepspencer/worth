use std::marker::PhantomData;

use super::{
    ApplicationWorkflowApprovalRef, ApplicationWorkflowAssessmentRef,
    ApplicationWorkflowComponentExpansion, ApplicationWorkflowCondition,
    ApplicationWorkflowDefinitionContentIdentity, ApplicationWorkflowDefinitionIdentity,
    ApplicationWorkflowDefinitionLimits, ApplicationWorkflowNodeIdentity,
    ApplicationWorkflowOperationRef, ApplicationWorkflowSpec, ApplicationWorkflowValidationDenial,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowNodeKind {
    Operation {
        operation: ApplicationWorkflowOperationRef,
        requires_workflow_authority: bool,
    },
    Assessment(ApplicationWorkflowAssessmentRef),
    Condition(ApplicationWorkflowCondition),
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
    pub const fn is_operation(&self) -> bool {
        matches!(self, Self::Operation { .. })
    }

    pub const fn is_effect(&self) -> bool {
        self.is_operation()
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
    /// Owner-published navigation over the settled path, never an authored control arm.
    NavigatedBack,
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
