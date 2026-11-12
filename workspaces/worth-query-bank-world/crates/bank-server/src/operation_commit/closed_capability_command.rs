//! Closed native command evidence and remaining cleanup/effect custody stay inseparable.
use super::{commit_receipt, BankCommitReceipt};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryCapabilityCommandCompletion as Completion, WorthQueryClosedOperation,
    },
    primary_graph::WorthQueryRecoveredApplicationCommitReceipt,
};

/// Owns the entire Query close tuple. Recorded meaning never becomes live capability.
#[must_use = "retain original native command and cleanup/effect custody"]
pub struct BankClosedCapabilityCommand {
    native: (WorthQueryClosedOperation, Completion),
    live_projection: Option<BankCommitReceipt>,
}
impl BankClosedCapabilityCommand {
    pub fn from_query(native: (WorthQueryClosedOperation, Completion)) -> Self {
        let live_projection = match &native.1 {
            Completion::Fresh { receipt } => Some(commit_receipt(receipt.clone())),
            Completion::DurableRecovered(_) | Completion::Recovered { .. } => None,
        };
        Self {
            native,
            live_projection,
        }
    }
    /// Borrows only the projection of an actually held live receipt.
    pub fn live_receipt(&self) -> Option<&BankCommitReceipt> {
        self.live_projection.as_ref()
    }
    pub fn recorded_receipt(&self) -> Option<&WorthQueryRecoveredApplicationCommitReceipt> {
        self.native.1.recovered_receipt()
    }
    pub fn completion(&self) -> &Completion {
        &self.native.1
    }
    pub fn closed(&self) -> &WorthQueryClosedOperation {
        &self.native.0
    }
    pub fn closed_mut(&mut self) -> &mut WorthQueryClosedOperation {
        &mut self.native.0
    }
    /// Transfers every original field and obligation; it extracts no receipt-only surrogate.
    pub fn into_query(self) -> (WorthQueryClosedOperation, Completion) {
        self.native
    }
}
impl std::fmt::Debug for BankClosedCapabilityCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BankClosedCapabilityCommand")
            .field("live_projection", &self.live_projection)
            .field("has_recorded_meaning", &self.recorded_receipt().is_some())
            .finish()
    }
}
