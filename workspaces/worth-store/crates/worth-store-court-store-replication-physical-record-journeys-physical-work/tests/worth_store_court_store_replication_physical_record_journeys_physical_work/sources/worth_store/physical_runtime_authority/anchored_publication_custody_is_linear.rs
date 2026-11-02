use worth_store::physical_runtime::{
    AnchoredRecordPublicationOperation, AnchoredRecordPublicationRecovery,
    AnchoredRecordStepCustodyPort, AnchoredRecordStepTerminal, AnchoredRecordStoppedPublication,
    AnchoredRecords, PhysicalMutationIdentity, PhysicalMutationOutcome,
};
fn copy_operation(operation: AnchoredRecordPublicationOperation) {
    let _ = operation.clone();
}
fn copy_recovery(recovery: AnchoredRecordPublicationRecovery) {
    let _ = recovery.clone();
}
fn expose_custody(operation: AnchoredRecordPublicationOperation) {
    let _ = operation.custody;
}
fn advance_twice(
    operation: AnchoredRecordPublicationOperation,
    records: &AnchoredRecords<'_>,
    port: &dyn AnchoredRecordStepCustodyPort,
) {
    let _ = operation.step(records, port);
    let _ = operation.step(records, port);
}
fn advance_stopped(
    stopped: AnchoredRecordStoppedPublication,
    records: &AnchoredRecords<'_>,
    port: &dyn AnchoredRecordStepCustodyPort,
) {
    let _ = stopped.step(records, port);
}
fn settle_twice(stopped: AnchoredRecordStoppedPublication, records: &AnchoredRecords<'_>) {
    let _ = stopped.settle(records);
    let _ = stopped.settle(records);
}
fn inject_terminal(mutation: PhysicalMutationIdentity, outcome: PhysicalMutationOutcome) {
    let _ = AnchoredRecordStepTerminal::new(mutation, outcome);
}
fn main() {}
