//! Known admission failure keeps the actual Home journal reopenable.
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::history::BranchId;

#[test]
fn known_checkpoint_failure_allows_close_and_reopen_preserves_the_journal_tail() {
    use worth_store_application::ApplicationHomeCloseOutcome;
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = crate::durable_run::open(directory.path(), &execution).unwrap();
    let committed = crate::workload::commit(
        home.relational().runtime(),
        &execution,
        &BranchId("main".into()),
        crate::batches::create(1),
    )
    .unwrap();
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    let expected = crate::workload::artifacts_of(&execution, home.relational().runtime());
    let closing = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(0)),
        CancellationSource::new().token(),
        None,
    );
    match home.close(&ExecutionRequest::serial(&closing)) {
        ApplicationHomeCloseOutcome::Closed { checkpoint, .. } => assert!(checkpoint.is_err()),
        ApplicationHomeCloseOutcome::CloseRefused { .. } => {
            panic!("a known checkpoint admission failure cannot refuse close")
        }
    }
    let home = crate::durable_run::open(directory.path(), &execution).unwrap();
    assert_eq!(
        crate::workload::artifacts_of(&execution, home.relational().runtime()),
        expected
    );
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}
