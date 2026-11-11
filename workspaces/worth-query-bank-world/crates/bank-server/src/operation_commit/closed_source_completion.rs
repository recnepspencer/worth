//! Original source result, recorded meaning and closed native custody remain owned.
use super::{commit_receipt, BankCommitReceipt};
use worth_query_host::facade::{
    application_entry::{WorthQueryClosedOperation, WorthQuerySourceCompletion as Completion},
    primary_graph::WorthQueryRecoveredApplicationCommitReceipt,
};

/// A closed source retains every native completion field, including its actual read/publication.
#[must_use = "retain original native source and cleanup/effect custody"]
pub struct BankClosedSourceCompletion<R> {
    native: (WorthQueryClosedOperation, Completion<R>),
    live_projection: Option<BankCommitReceipt>,
}
impl<R> BankClosedSourceCompletion<R> {
    pub fn from_query(native: (WorthQueryClosedOperation, Completion<R>)) -> Self {
        let live_projection = match &native.1 {
            Completion::Fresh { receipt, .. } | Completion::Recovered { receipt, .. } => {
                Some(commit_receipt(receipt.clone()))
            }
            Completion::Recorded { .. } => None,
        };
        Self {
            native,
            live_projection,
        }
    }
    pub fn live_receipt(&self) -> Option<&BankCommitReceipt> {
        self.live_projection.as_ref()
    }
    pub fn recorded_receipt(&self) -> Option<&WorthQueryRecoveredApplicationCommitReceipt> {
        self.native.1.recorded_receipt()
    }
    pub fn fresh_result(&self) -> Option<&R> {
        match &self.native.1 {
            Completion::Fresh { result, .. } => Some(result),
            Completion::Recovered { .. } | Completion::Recorded { .. } => None,
        }
    }
    pub fn completion(&self) -> &Completion<R> {
        &self.native.1
    }
    pub fn closed(&self) -> &WorthQueryClosedOperation {
        &self.native.0
    }
    pub fn closed_mut(&mut self) -> &mut WorthQueryClosedOperation {
        &mut self.native.0
    }
    pub fn into_query(self) -> (WorthQueryClosedOperation, Completion<R>) {
        self.native
    }
}
impl<R> std::fmt::Debug for BankClosedSourceCompletion<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BankClosedSourceCompletion")
            .field("live_projection", &self.live_projection)
            .field("has_recorded_meaning", &self.recorded_receipt().is_some())
            .finish()
    }
}
