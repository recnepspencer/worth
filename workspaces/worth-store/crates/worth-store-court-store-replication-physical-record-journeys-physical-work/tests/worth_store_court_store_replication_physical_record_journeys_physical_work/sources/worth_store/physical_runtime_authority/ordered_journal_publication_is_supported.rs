//! Compile-body counterparts take already-owned values; no runtime or SDK grant is minted.
use worth_store::physical_runtime::{
    OrderedJournal, OrderedJournalCommitted, OrderedJournalEntry, OrderedJournalPublication,
    OrderedJournalPublicationOperation, OrderedJournalPublicationRecovery,
    OrderedJournalPublicationResume, OrderedJournalPublicationStep, OrderedJournalStoppedPoll,
    OrderedJournalStoppedPublication,
};
fn advance(
    operation: OrderedJournalPublicationOperation,
    journal: &OrderedJournal<'_>,
) -> OrderedJournalPublicationStep {
    operation.step(journal)
}
fn stop(operation: OrderedJournalPublicationOperation) -> OrderedJournalStoppedPublication {
    operation.stop()
}
fn latch(recovery: OrderedJournalPublicationRecovery) -> OrderedJournalPublicationRecovery {
    recovery.stop()
}
fn ready_pair(
    recovery: OrderedJournalPublicationRecovery,
) -> Option<(
    OrderedJournalPublicationOperation,
    OrderedJournalPublicationRecovery,
)> {
    match recovery.resume() {
        OrderedJournalPublicationResume::Ready {
            operation,
            recovery,
        } => Some((operation, recovery)),
        _ => None,
    }
}
fn recovered(recovery: OrderedJournalPublicationRecovery) -> Option<OrderedJournalCommitted> {
    match recovery.resume() {
        OrderedJournalPublicationResume::Committed(actual) => Some(actual),
        _ => None,
    }
}
fn settle(
    stopped: OrderedJournalStoppedPublication,
    journal: &OrderedJournal<'_>,
) -> OrderedJournalStoppedPoll {
    stopped.settle(journal)
}
fn entry(committed: OrderedJournalCommitted) -> OrderedJournalEntry {
    committed.entry()
}
pub fn main() {
    std::hint::black_box(advance);
    std::hint::black_box(stop);
    std::hint::black_box(latch);
    std::hint::black_box(ready_pair);
    std::hint::black_box(recovered);
    std::hint::black_box(settle);
    std::hint::black_box(entry);
    std::hint::black_box(OrderedJournalPublication::with_input_memory);
    std::hint::black_box(OrderedJournal::start_publication);
}
