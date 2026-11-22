//! One exact Native receipt closure shared by the actual coupled actors.
use super::*;
use worth_relational::facade::mvcc::ValidatedRelationalProposal;

/// Borrows the one immutable candidate whose complete invocation set was closed.
/// Members can inspect this candidate; they cannot substitute another judgment.
pub struct ClosedCoupledInvariantReceipts<'candidate> {
    candidate: &'candidate ValidatedRelationalProposal,
    native_work: u64,
}
impl<'candidate> ClosedCoupledInvariantReceipts<'candidate> {
    pub fn admit(
        candidate: &'candidate ValidatedRelationalProposal,
        requirements: &[WorthQueryInstalledInvariantExecutionRequirement],
    ) -> Result<Self, WorthQueryInvariantExecutionFailure> {
        let native_work =
            validate_receipt_closure(candidate.invariant_evidence(), requirements, 0)?;
        Ok(Self {
            candidate,
            native_work,
        })
    }
    pub fn candidate(&self) -> &'candidate ValidatedRelationalProposal {
        self.candidate
    }
    pub fn native_work(&self) -> u64 {
        self.native_work
    }
}
