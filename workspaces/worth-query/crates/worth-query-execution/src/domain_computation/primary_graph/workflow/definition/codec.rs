use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowConnectionKind, ApplicationWorkflowControlOutcome,
    ApplicationWorkflowDataFlow, ApplicationWorkflowNodeKind,
};

#[derive(Clone, Copy)]
pub(super) enum WorkflowNodeTag {
    Operation,
    Assessment,
    Approval,
    EvidenceJoin,
    Terminal,
}

impl WorkflowNodeTag {
    pub(super) const fn from_declared(kind: &ApplicationWorkflowNodeKind) -> Self {
        match kind {
            ApplicationWorkflowNodeKind::Operation { .. } => Self::Operation,
            ApplicationWorkflowNodeKind::Assessment(_) => Self::Assessment,
            ApplicationWorkflowNodeKind::Approval(_) => Self::Approval,
            ApplicationWorkflowNodeKind::EvidenceJoin => Self::EvidenceJoin,
            ApplicationWorkflowNodeKind::Terminal => Self::Terminal,
        }
    }

    pub(super) const fn from_persisted(value: u64) -> Option<Self> {
        match value {
            0 => Some(Self::Operation),
            1 => Some(Self::Assessment),
            2 => Some(Self::Approval),
            3 => Some(Self::EvidenceJoin),
            4 => Some(Self::Terminal),
            _ => None,
        }
    }

    pub(super) const fn persisted(self) -> u64 {
        match self {
            Self::Operation => 0,
            Self::Assessment => 1,
            Self::Approval => 2,
            Self::EvidenceJoin => 3,
            Self::Terminal => 4,
        }
    }

    pub(super) const fn identity(self) -> &'static str {
        match self {
            Self::Operation => "operation",
            Self::Assessment => "assessment",
            Self::Approval => "approval",
            Self::EvidenceJoin => "evidence-join",
            Self::Terminal => "terminal",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum WorkflowConnectionTag {
    Control(ApplicationWorkflowControlOutcome),
    Data(ApplicationWorkflowDataFlow),
}

impl WorkflowConnectionTag {
    pub(super) const fn from_declared(kind: ApplicationWorkflowConnectionKind) -> Self {
        match kind {
            ApplicationWorkflowConnectionKind::Control(outcome) => Self::Control(outcome),
            ApplicationWorkflowConnectionKind::Data(flow) => Self::Data(flow),
        }
    }

    pub(super) const fn control(outcome: ApplicationWorkflowControlOutcome) -> Self {
        Self::Control(outcome)
    }

    pub(super) const fn data(flow: ApplicationWorkflowDataFlow) -> Self {
        Self::Data(flow)
    }

    pub(super) const fn from_persisted(family: u64, variant: u64) -> Option<Self> {
        match (family, variant) {
            (0, 0) => Some(Self::Control(ApplicationWorkflowControlOutcome::Completed)),
            (0, 1) => Some(Self::Control(ApplicationWorkflowControlOutcome::Approved)),
            (0, 2) => Some(Self::Control(ApplicationWorkflowControlOutcome::Rejected)),
            (1, 0) => Some(Self::Data(ApplicationWorkflowDataFlow::ProposalSubject)),
            (1, 1) => Some(Self::Data(ApplicationWorkflowDataFlow::AssessmentSubject)),
            (1, 2) => Some(Self::Data(ApplicationWorkflowDataFlow::AssessmentEvidence)),
            (1, 3) => Some(Self::Data(ApplicationWorkflowDataFlow::JoinedEvidence)),
            (1, 4) => Some(Self::Data(ApplicationWorkflowDataFlow::ApprovalAuthority)),
            (1, 5) => Some(Self::Data(ApplicationWorkflowDataFlow::OperationInput)),
            _ => None,
        }
    }

    pub(super) const fn family(self) -> u64 {
        match self {
            Self::Control(_) => 0,
            Self::Data(_) => 1,
        }
    }

    pub(super) const fn variant(self) -> u64 {
        match self {
            Self::Control(ApplicationWorkflowControlOutcome::Completed)
            | Self::Data(ApplicationWorkflowDataFlow::ProposalSubject) => 0,
            Self::Control(ApplicationWorkflowControlOutcome::Approved)
            | Self::Data(ApplicationWorkflowDataFlow::AssessmentSubject) => 1,
            Self::Control(ApplicationWorkflowControlOutcome::Rejected)
            | Self::Data(ApplicationWorkflowDataFlow::AssessmentEvidence) => 2,
            Self::Data(ApplicationWorkflowDataFlow::JoinedEvidence) => 3,
            Self::Data(ApplicationWorkflowDataFlow::ApprovalAuthority) => 4,
            Self::Data(ApplicationWorkflowDataFlow::OperationInput) => 5,
        }
    }

    pub(super) const fn identity(self) -> &'static str {
        match self {
            Self::Control(ApplicationWorkflowControlOutcome::Completed) => "control-completed",
            Self::Control(ApplicationWorkflowControlOutcome::Approved) => "control-approved",
            Self::Control(ApplicationWorkflowControlOutcome::Rejected) => "control-rejected",
            Self::Data(ApplicationWorkflowDataFlow::ProposalSubject) => "data-proposal-subject",
            Self::Data(ApplicationWorkflowDataFlow::AssessmentSubject) => "data-assessment-subject",
            Self::Data(ApplicationWorkflowDataFlow::AssessmentEvidence) => {
                "data-assessment-evidence"
            }
            Self::Data(ApplicationWorkflowDataFlow::JoinedEvidence) => "data-joined-evidence",
            Self::Data(ApplicationWorkflowDataFlow::ApprovalAuthority) => "data-approval-authority",
            Self::Data(ApplicationWorkflowDataFlow::OperationInput) => "data-operation-input",
        }
    }
}
