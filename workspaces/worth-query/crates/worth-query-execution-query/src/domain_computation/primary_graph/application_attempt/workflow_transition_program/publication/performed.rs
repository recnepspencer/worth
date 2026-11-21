use super::{PerformedWorkflowAssessmentEvidence, WorkflowApprovalDecision};

/// A workflow transition that landed: the node it reached, whether that node is terminal,
/// its commit receipt, whether it replayed, and the assessment evidence, approval or
/// operation receipt it consumed.
#[derive(Debug)]
pub struct PerformedWorkflowTransition {
    pub(super) transition: worth_relational::facade::identity::EntityId,
    pub(super) node_path: String,
    pub(super) terminal: bool,
    pub(super) receipt:
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    pub(super) replayed: bool,
    pub(super) assessment_evidence: Option<PerformedWorkflowAssessmentEvidence>,
    pub(super) approval: Option<PerformedWorkflowApproval>,
    pub(super) operation_receipt_identity: Option<[u8; 32]>,
}

/// An approval recorded by a transition: its decision, the proposal and evidence it
/// covered, the approver, its purpose and its expiry.
#[derive(Debug)]
pub struct PerformedWorkflowApproval {
    pub(super) entity: worth_relational::facade::identity::EntityId,
    pub(super) identity: String,
    pub(super) decision: WorkflowApprovalDecision,
    pub(super) proposal: worth_relational::facade::identity::EntityId,
    pub(super) evidence: Box<[worth_relational::facade::identity::EntityId]>,
    pub(super) approver: worth_relational::facade::identity::EntityId,
    pub(super) purpose: String,
    pub(super) expiry: u64,
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

    pub const fn receipt(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt {
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
