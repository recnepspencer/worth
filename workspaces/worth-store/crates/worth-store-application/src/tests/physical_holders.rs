//! Bisect real physical ownership at Home lifecycle boundaries, without a Bank server.
use super::lifecycle::open;
use crate::facade::ApplicationHomeCloseOutcome;
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn physical_holders_leave_no_owner_after_fresh_and_reopened_home_close() {
    let directory = tempfile::tempdir().unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    for cycle in ["fresh", "reopened"] {
        let home = open(directory.path(), &execution);
        let physical = Arc::downgrade(&home.context.serving);
        eprintln!(
            "HOLDER_PROBE {cycle} after-open {}",
            physical.strong_count()
        );
        super::native_schema::commit(home.relational().runtime(), &execution, cycle);
        eprintln!(
            "HOLDER_PROBE {cycle} after-native-commit {}",
            physical.strong_count()
        );
        let outcome = home.close(&execution);
        eprintln!(
            "HOLDER_PROBE {cycle} after-close {}",
            physical.strong_count()
        );
        match outcome {
            ApplicationHomeCloseOutcome::Closed { .. } => {}
            ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
                panic!("{cycle} close refused: {reason:?}");
            }
        }
        assert_eq!(physical.strong_count(), 0, "{cycle} physical owner retired");
    }
}
