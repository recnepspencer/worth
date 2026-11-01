//! Compile-body witnesses only: no Store construction or runtime completion claim.
use worth_store::physical_runtime::{
    AnchoredRecordPublicationObservation, AnchoredRecordPublicationOperation,
    AnchoredRecordPublicationRecovery, AnchoredRecordPublicationResume,
    AnchoredRecordPublicationStep, AnchoredRecordStepCustodyPort, AnchoredRecordStepTerminal,
    AnchoredRecordStoppedPoll, AnchoredRecordStoppedPublication, AnchoredRecords,
    PhysicalMutationOutcome,
};
fn observe(operation: &AnchoredRecordPublicationOperation) -> AnchoredRecordPublicationObservation {
    operation.observe()
}
fn advance(
    operation: AnchoredRecordPublicationOperation,
    records: &AnchoredRecords<'_>,
    port: &dyn AnchoredRecordStepCustodyPort,
) -> AnchoredRecordPublicationStep {
    operation.step(records, port)
}
fn stop(operation: AnchoredRecordPublicationOperation) -> AnchoredRecordStoppedPublication {
    operation.stop()
}
fn resume(recovery: AnchoredRecordPublicationRecovery) -> AnchoredRecordPublicationResume {
    recovery.resume()
}
fn settle(
    stopped: AnchoredRecordStoppedPublication,
    records: &AnchoredRecords<'_>,
) -> AnchoredRecordStoppedPoll {
    stopped.settle(records)
}
fn evidence(terminal: &AnchoredRecordStepTerminal) -> &PhysicalMutationOutcome {
    terminal.outcome()
}
pub fn main() {
    std::hint::black_box(observe);
    std::hint::black_box(advance);
    std::hint::black_box(stop);
    std::hint::black_box(resume);
    std::hint::black_box(settle);
    std::hint::black_box(evidence);
}
