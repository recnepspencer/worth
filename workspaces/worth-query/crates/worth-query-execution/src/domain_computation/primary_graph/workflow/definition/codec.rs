use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowConnectionKind, ApplicationWorkflowControlOutcome,
    ApplicationWorkflowDataFlow, ApplicationWorkflowNodeKind,
};

#[derive(Clone, Copy)]
pub(super) enum WorkflowNodeTag {
    Operation,
    Assessment,
    Condition,
    Approval,
    EvidenceJoin,
    Terminal,
}

impl WorkflowNodeTag {
    pub(super) const fn from_declared(kind: &ApplicationWorkflowNodeKind) -> Self {
        match kind {
            ApplicationWorkflowNodeKind::Operation { .. } => Self::Operation,
            ApplicationWorkflowNodeKind::Assessment(_) => Self::Assessment,
            ApplicationWorkflowNodeKind::Condition(_) => Self::Condition,
            ApplicationWorkflowNodeKind::Approval(_) => Self::Approval,
            ApplicationWorkflowNodeKind::EvidenceJoin(_) => Self::EvidenceJoin,
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
            5 => Some(Self::Condition),
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
            Self::Condition => 5,
        }
    }

    pub(super) const fn identity(self) -> &'static str {
        match self {
            Self::Operation => "operation",
            Self::Assessment => "assessment",
            Self::Approval => "approval",
            Self::EvidenceJoin => "evidence-join",
            Self::Terminal => "terminal",
            Self::Condition => "condition",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum WorkflowConnectionTag {
    Control(ApplicationWorkflowControlOutcome),
    Data(ApplicationWorkflowDataFlow),
    Retry(ApplicationWorkflowControlOutcome),
}

impl WorkflowConnectionTag {
    pub(super) fn from_declared(kind: ApplicationWorkflowConnectionKind) -> Self {
        match kind {
            ApplicationWorkflowConnectionKind::Control(outcome) => Self::Control(outcome),
            ApplicationWorkflowConnectionKind::Data(flow) => Self::Data(flow),
            ApplicationWorkflowConnectionKind::Retry(retry) => Self::Retry(retry.trigger()),
        }
    }

    pub(super) const fn control(outcome: ApplicationWorkflowControlOutcome) -> Self {
        Self::Control(outcome)
    }

    pub(super) const fn data(flow: ApplicationWorkflowDataFlow) -> Self {
        Self::Data(flow)
    }

    pub(super) fn from_persisted(family: u64, variant: u64) -> Option<Self> {
        match (family, variant) {
            (0, 0) => Some(Self::Control(ApplicationWorkflowControlOutcome::Completed)),
            (0, 1) => Some(Self::Control(ApplicationWorkflowControlOutcome::Approved)),
            (0, 2) => Some(Self::Control(ApplicationWorkflowControlOutcome::Rejected)),
            (0, 3) => Some(Self::Control(
                ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            )),
            (0, 4) => Some(Self::Control(
                ApplicationWorkflowControlOutcome::EvidenceFailed,
            )),
            (0, 5) => Some(Self::Control(
                ApplicationWorkflowControlOutcome::RetryExhausted,
            )),
            (0, 6) => Some(Self::Control(
                ApplicationWorkflowControlOutcome::ConditionSatisfied,
            )),
            (0, 7) => Some(Self::Control(
                ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
            )),
            (0, 8) => Some(Self::Control(
                ApplicationWorkflowControlOutcome::NavigatedBack,
            )),
            (1, 0) => Some(Self::Data(ApplicationWorkflowDataFlow::ProposalSubject)),
            (1, 1) => Some(Self::Data(ApplicationWorkflowDataFlow::AssessmentSubject)),
            (1, 2) => Some(Self::Data(ApplicationWorkflowDataFlow::AssessmentEvidence)),
            (1, 3) => Some(Self::Data(ApplicationWorkflowDataFlow::JoinedEvidence)),
            (1, 4) => Some(Self::Data(ApplicationWorkflowDataFlow::ApprovalAuthority)),
            (1, 5) => Some(Self::Data(ApplicationWorkflowDataFlow::OperationInput)),
            (1, 6) => Some(Self::Data(ApplicationWorkflowDataFlow::ConditionSubject)),
            (2, variant) => decode_control(variant).map(Self::Retry),
            _ => None,
        }
    }

    pub(super) const fn family(self) -> u64 {
        match self {
            Self::Control(_) => 0,
            Self::Data(_) => 1,
            Self::Retry(_) => 2,
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
            Self::Control(ApplicationWorkflowControlOutcome::EvidenceSatisfied)
            | Self::Data(ApplicationWorkflowDataFlow::JoinedEvidence) => 3,
            Self::Control(ApplicationWorkflowControlOutcome::EvidenceFailed)
            | Self::Data(ApplicationWorkflowDataFlow::ApprovalAuthority) => 4,
            Self::Data(ApplicationWorkflowDataFlow::OperationInput) => 5,
            Self::Control(ApplicationWorkflowControlOutcome::RetryExhausted) => 5,
            Self::Control(ApplicationWorkflowControlOutcome::ConditionSatisfied) => 6,
            Self::Control(ApplicationWorkflowControlOutcome::ConditionUnsatisfied) => 7,
            Self::Control(ApplicationWorkflowControlOutcome::NavigatedBack) => 8,
            Self::Data(ApplicationWorkflowDataFlow::ConditionSubject) => 6,
            Self::Retry(outcome) => control_variant(outcome),
        }
    }

    pub(super) const fn identity(self) -> &'static str {
        match self {
            Self::Control(ApplicationWorkflowControlOutcome::Completed) => "control-completed",
            Self::Control(ApplicationWorkflowControlOutcome::Approved) => "control-approved",
            Self::Control(ApplicationWorkflowControlOutcome::Rejected) => "control-rejected",
            Self::Control(ApplicationWorkflowControlOutcome::EvidenceSatisfied) => {
                "control-evidence-satisfied"
            }
            Self::Control(ApplicationWorkflowControlOutcome::EvidenceFailed) => {
                "control-evidence-failed"
            }
            Self::Control(ApplicationWorkflowControlOutcome::RetryExhausted) => {
                "control-retry-exhausted"
            }
            Self::Control(ApplicationWorkflowControlOutcome::ConditionSatisfied) => {
                "control-condition-satisfied"
            }
            Self::Control(ApplicationWorkflowControlOutcome::ConditionUnsatisfied) => {
                "control-condition-unsatisfied"
            }
            Self::Control(ApplicationWorkflowControlOutcome::NavigatedBack) => {
                "control-navigated-back"
            }
            Self::Data(ApplicationWorkflowDataFlow::ProposalSubject) => "data-proposal-subject",
            Self::Data(ApplicationWorkflowDataFlow::AssessmentSubject) => "data-assessment-subject",
            Self::Data(ApplicationWorkflowDataFlow::AssessmentEvidence) => {
                "data-assessment-evidence"
            }
            Self::Data(ApplicationWorkflowDataFlow::JoinedEvidence) => "data-joined-evidence",
            Self::Data(ApplicationWorkflowDataFlow::ApprovalAuthority) => "data-approval-authority",
            Self::Data(ApplicationWorkflowDataFlow::OperationInput) => "data-operation-input",
            Self::Data(ApplicationWorkflowDataFlow::ConditionSubject) => "data-condition-subject",
            Self::Retry(_) => "retry",
        }
    }
}

const fn control_variant(outcome: ApplicationWorkflowControlOutcome) -> u64 {
    match outcome {
        ApplicationWorkflowControlOutcome::Completed => 0,
        ApplicationWorkflowControlOutcome::Approved => 1,
        ApplicationWorkflowControlOutcome::Rejected => 2,
        ApplicationWorkflowControlOutcome::EvidenceSatisfied => 3,
        ApplicationWorkflowControlOutcome::EvidenceFailed => 4,
        ApplicationWorkflowControlOutcome::RetryExhausted => 5,
        ApplicationWorkflowControlOutcome::ConditionSatisfied => 6,
        ApplicationWorkflowControlOutcome::ConditionUnsatisfied => 7,
        ApplicationWorkflowControlOutcome::NavigatedBack => 8,
    }
}

const fn decode_control(variant: u64) -> Option<ApplicationWorkflowControlOutcome> {
    match variant {
        0 => Some(ApplicationWorkflowControlOutcome::Completed),
        1 => Some(ApplicationWorkflowControlOutcome::Approved),
        2 => Some(ApplicationWorkflowControlOutcome::Rejected),
        3 => Some(ApplicationWorkflowControlOutcome::EvidenceSatisfied),
        4 => Some(ApplicationWorkflowControlOutcome::EvidenceFailed),
        5 => Some(ApplicationWorkflowControlOutcome::RetryExhausted),
        6 => Some(ApplicationWorkflowControlOutcome::ConditionSatisfied),
        7 => Some(ApplicationWorkflowControlOutcome::ConditionUnsatisfied),
        8 => Some(ApplicationWorkflowControlOutcome::NavigatedBack),
        _ => None,
    }
}
