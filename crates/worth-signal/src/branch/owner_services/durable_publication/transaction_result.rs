//! The original transaction result retains its admitted outer allocation.
use crate::logic::transaction::TransactionResult;
pub struct SignalDurableTransactionResult {
    pub(super) result: Box<TransactionResult>,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl SignalDurableTransactionResult {
    pub fn result(&self) -> &TransactionResult {
        &self.result
    }
}
