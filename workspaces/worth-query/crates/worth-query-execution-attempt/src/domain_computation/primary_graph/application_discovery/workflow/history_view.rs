//! Descriptive bounded transition, proposal and approval values, never command evidence.
use super::{WorthQueryWorkflowReadBasis, WorthQueryWorkflowRecordedActor};
use crate::domain_computation::primary_graph::application_attempt::PublishedWorkflowProposalRef;
use worth_execution::{ExecutionArray, ExecutionImmutableBytes};
use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
/// An approval at the exact data basis, not permission to perform the proposed operation.
pub struct WorthQueryWorkflowApprovalRead {
    pub(crate) decision:
        crate::domain_computation::primary_graph::application_attempt::WorkflowApprovalDecision,
    pub(crate) proposal_identity: ExecutionImmutableBytes,
}
impl WorthQueryWorkflowApprovalRead {
    pub fn decision(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorkflowApprovalDecision
    {
        self.decision
    }
    pub fn proposal_identity(&self) -> &str {
        text(&self.proposal_identity)
    }
}
/// Audit withholding does not mean that a role or a lifecycle event is absent.
pub enum WorthQueryWorkflowHistoryAudit {
    Withheld,
    Visible {
        advancer: WorthQueryWorkflowRecordedActor,
        executor: Option<WorthQueryWorkflowRecordedActor>,
        proposer: Option<WorthQueryWorkflowRecordedActor>,
        approver: Option<WorthQueryWorkflowRecordedActor>,
    },
}
/// One canonical recorded transition, projected without advancing or warming progress.
pub struct WorthQueryWorkflowHistoryEntry {
    pub(crate) occurrence: u64,
    pub(crate) path: ExecutionImmutableBytes,
    pub(crate) outcome: ApplicationWorkflowControlOutcome,
    pub(crate) proposal: Option<PublishedWorkflowProposalRef>,
    pub(crate) approval: Option<WorthQueryWorkflowApprovalRead>,
    pub(crate) audit: WorthQueryWorkflowHistoryAudit,
}
impl WorthQueryWorkflowHistoryEntry {
    pub fn occurrence(&self) -> u64 {
        self.occurrence
    }
    pub fn node_path(&self) -> &str {
        text(&self.path)
    }
    pub fn outcome(&self) -> ApplicationWorkflowControlOutcome {
        self.outcome
    }
    /// Reacquired native locator. Pendingness and command legality are rechecked by the command owner.
    pub fn proposal(&self) -> Option<&PublishedWorkflowProposalRef> {
        self.proposal.as_ref()
    }
    pub fn approval(&self) -> Option<&WorthQueryWorkflowApprovalRead> {
        self.approval.as_ref()
    }
    pub fn audit(&self) -> &WorthQueryWorkflowHistoryAudit {
        &self.audit
    }
}
fn text(value: &ExecutionImmutableBytes) -> &str {
    std::str::from_utf8(value.bytes()).expect("native UTF-8 projection")
}
/// A complete page and its opaque continuation, retaining exact data but no grant.
pub struct WorthQueryWorkflowHistoryPage {
    pub(crate) basis: WorthQueryWorkflowReadBasis,
    pub(crate) instance: worth_relational::facade::identity::EntityId,
    pub(crate) subject: worth_relational::facade::identity::EntityId,
    pub(crate) scope: worth_relational::facade::identity::EntityId,
    pub(crate) support: [u8; 32],
    pub(crate) width: usize,
    pub(crate) rows: ExecutionArray<WorthQueryWorkflowHistoryEntry>,
    pub(crate) native: worth_relational::facade::indexes::BoundedRelatedEntityPrefixPage,
}
impl WorthQueryWorkflowHistoryPage {
    pub fn basis(&self) -> &WorthQueryWorkflowReadBasis {
        &self.basis
    }
    pub fn entries(&self) -> &[WorthQueryWorkflowHistoryEntry] {
        self.rows.elements()
    }
    pub fn continuation(&self) -> Option<&Self> {
        self.native.next_boundary().map(|_| self)
    }
}
