use worth_store_recovery_physics::PageLsn;
use worth_store_recovery_runtime::{complete_recovery, RecoveryCompletion};
use worth_store_wal::LogSequenceNumber;

/// Supplies descriptive terminal facts for consumers of the runtime completion boundary.
///
/// This fixture does not mint Store authority or claim that a Store recovery was executed.
pub fn recovery_completion() -> RecoveryCompletion {
    recovery_completion_with_operation_digest("op-20")
}

pub fn recovery_completion_with_operation_digest(operation_digest: &str) -> RecoveryCompletion {
    let page_lsn = PageLsn::from_lsn(LogSequenceNumber::new(20));
    complete_recovery(
        format!("recovered-root:{operation_digest}"),
        Some(page_lsn),
        1,
        2,
        format!("source-decision:{operation_digest}"),
    )
    .expect("recovery completion fixture contains non-empty descriptive facts")
}
