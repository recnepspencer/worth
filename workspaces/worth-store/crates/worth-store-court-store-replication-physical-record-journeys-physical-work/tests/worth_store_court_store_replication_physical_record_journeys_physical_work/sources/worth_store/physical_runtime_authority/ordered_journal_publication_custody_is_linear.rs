//! Adversarial owned endpoint reuse and caller replacement of actual receipt custody.
use std::sync::Arc;
use worth_store::physical_runtime::{
    OrderedJournal, OrderedJournalCommitted, OrderedJournalPublicationOperation,
    OrderedJournalPublicationRecovery, OrderedJournalPublicationTerminal,
    OrderedJournalStoppedPublication,
};
fn clone_operation(operation: OrderedJournalPublicationOperation) {
    let _ = operation.clone();
}
fn clone_recovery(recovery: OrderedJournalPublicationRecovery) {
    let _ = recovery.clone();
}
fn advance_stopped(stopped: OrderedJournalStoppedPublication, journal: &OrderedJournal<'_>) {
    let _ = stopped.step(journal);
}
fn step_twice(operation: OrderedJournalPublicationOperation, journal: &OrderedJournal<'_>) {
    let _ = operation.step(journal);
    let _ = operation.step(journal);
}
fn settle_twice(stopped: OrderedJournalStoppedPublication, journal: &OrderedJournal<'_>) {
    let _ = stopped.settle(journal);
    let _ = stopped.settle(journal);
}
fn resume_twice(recovery: OrderedJournalPublicationRecovery) {
    let _ = recovery.resume();
    let _ = recovery.resume();
}
fn replace_actual_receipt(
    mut committed: OrderedJournalCommitted,
    caller_terminal: Arc<OrderedJournalPublicationTerminal>,
) {
    committed.terminal = caller_terminal;
}
fn main() {}
