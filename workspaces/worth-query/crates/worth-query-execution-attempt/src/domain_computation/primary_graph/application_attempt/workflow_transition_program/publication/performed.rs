use super::{PerformedWorkflowAssessmentEvidence, WorkflowApprovalDecision};

/// A workflow transition that landed: the node it reached, whether that node is terminal,
/// its commit receipt, whether it replayed, and the assessment evidence, approval or
/// operation receipt it consumed.
#[derive(Debug)]
pub struct PerformedWorkflowTransition {
    transition: worth_relational::facade::identity::EntityId,
    node_path: String,
    terminal: bool,
    receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    replayed: bool,
    assessment_evidence: Option<PerformedWorkflowAssessmentEvidence>,
    approval: Option<PerformedWorkflowApproval>,
    operation_receipt_identity: Option<[u8; 32]>,
}

/// An approval recorded by a transition: its decision, the proposal and evidence it
/// covered, the approver, its purpose and its expiry.
#[derive(Debug)]
pub struct PerformedWorkflowApproval {
    entity: worth_relational::facade::identity::EntityId,
    identity: String,
    decision: WorkflowApprovalDecision,
    proposal: worth_relational::facade::identity::EntityId,
    evidence: Box<[worth_relational::facade::identity::EntityId]>,
    approver: worth_relational::facade::identity::EntityId,
    purpose: String,
    expiry: u64,
}

impl PerformedWorkflowApproval {
    pub const fn entity(&self) -> worth_relational::facade::identity::EntityId {
        self.entity
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub const fn decision(&self) -> WorkflowApprovalDecision {
        self.decision
    }
    pub const fn proposal(&self) -> worth_relational::facade::identity::EntityId {
        self.proposal
    }
    pub fn evidence(&self) -> &[worth_relational::facade::identity::EntityId] {
        &self.evidence
    }
    pub const fn approver(&self) -> worth_relational::facade::identity::EntityId {
        self.approver
    }
    pub fn purpose(&self) -> &str {
        &self.purpose
    }
    pub const fn expiry(&self) -> u64 {
        self.expiry
    }
}

impl PerformedWorkflowTransition {
    pub const fn transition(&self) -> worth_relational::facade::identity::EntityId {
        self.transition
    }

    pub fn node_path(&self) -> &str {
        &self.node_path
    }

    pub const fn terminal(&self) -> bool {
        self.terminal
    }

    pub const fn receipt(&self) -> &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn replayed(&self) -> bool {
        self.replayed
    }

    pub const fn assessment_evidence(&self) -> Option<&PerformedWorkflowAssessmentEvidence> {
        self.assessment_evidence.as_ref()
    }

    pub const fn approval(&self) -> Option<&PerformedWorkflowApproval> {
        self.approval.as_ref()
    }

    pub const fn operation_receipt_identity(&self) -> Option<&[u8; 32]> {
        self.operation_receipt_identity.as_ref()
    }
}


#[doc(hidden)]
pub struct PerformedWorkflowTransitionAttemptParts {
    pub transition: worth_relational::facade::identity::EntityId,
    pub node_path: String,
    pub terminal: bool,
    pub receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    pub replayed: bool,
    pub assessment_evidence: Option<PerformedWorkflowAssessmentEvidence>,
    pub approval: Option<PerformedWorkflowApproval>,
    pub operation_receipt_identity: Option<[u8; 32]>,
}

impl PerformedWorkflowTransition {
    #[doc(hidden)]
    pub fn from_attempt_parts(
        _seal: crate::ports::WorthQueryAttemptCrateSeal,
        parts: PerformedWorkflowTransitionAttemptParts,
    ) -> Self {
        Self {
            transition: parts.transition,
            node_path: parts.node_path,
            terminal: parts.terminal,
            receipt: parts.receipt,
            replayed: parts.replayed,
            assessment_evidence: parts.assessment_evidence,
            approval: parts.approval,
            operation_receipt_identity: parts.operation_receipt_identity,
        }
    }
}

#[doc(hidden)]
pub struct PerformedWorkflowApprovalAttemptParts {
    pub entity: worth_relational::facade::identity::EntityId,
    pub identity: String,
    pub decision: WorkflowApprovalDecision,
    pub proposal: worth_relational::facade::identity::EntityId,
    pub evidence: Box<[worth_relational::facade::identity::EntityId]>,
    pub approver: worth_relational::facade::identity::EntityId,
    pub purpose: String,
    pub expiry: u64,
}

impl PerformedWorkflowApproval {
    #[doc(hidden)]
    pub fn from_attempt_parts(
        _seal: crate::ports::WorthQueryAttemptCrateSeal,
        parts: PerformedWorkflowApprovalAttemptParts,
    ) -> Self {
        Self {
            entity: parts.entity,
            identity: parts.identity,
            decision: parts.decision,
            proposal: parts.proposal,
            evidence: parts.evidence,
            approver: parts.approver,
            purpose: parts.purpose,
            expiry: parts.expiry,
        }
    }
}
