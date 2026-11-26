//! Issue #423: committed WAL history beyond the pending-unresolved bound reopens.
use crate::facade::ApplicationHomeCloseOutcome;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

/// A native-only Home cannot publish a Home checkpoint, so each commit's one
/// physical WAL member stays in the retained tail across a clean close. The
/// production profile bounds pending-unresolved bindings at 1024.
const COMMITS: usize = 1030;

#[test]
fn clean_close_after_more_uncheckpointed_wal_members_than_the_pending_bound_reopens() {
    let directory = tempfile::tempdir().unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let home = super::lifecycle::open(directory.path(), &execution);
    for index in 0..COMMITS {
        super::native_schema::commit(
            home.relational().runtime(),
            &execution,
            &format!("uncheckpointed-{index}"),
        );
    }
    let before = super::native_schema::artifacts(&execution, home.relational().runtime());
    assert_eq!(before.len(), COMMITS);
    match home.close(&execution) {
        ApplicationHomeCloseOutcome::Closed { checkpoint, .. } => assert!(matches!(
            checkpoint,
            Err(crate::ApplicationHomeCheckpointDenial::WorldSectionRequired)
        )),
        ApplicationHomeCloseOutcome::CloseRefused { .. } => panic!("quiescent owner must close"),
    }
    let home = super::lifecycle::open(directory.path(), &execution);
    assert_eq!(
        super::native_schema::artifacts(&execution, home.relational().runtime()),
        before
    );
    // The reopened history holds no pending capacity: fresh admission proceeds.
    super::native_schema::commit(home.relational().runtime(), &execution, "after-reopen");
    assert_eq!(
        super::native_schema::artifacts(&execution, home.relational().runtime()).len(),
        COMMITS + 1
    );
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}
