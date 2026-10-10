//! Owned observation scope shared by real checkpoint recovery probes.
use super::primary_graph;

pub(super) struct CheckpointRecoveryTestGuard {
    _query_observation: primary_graph::WorthQueryExecutionObservation,
    _exclusive: std::sync::MutexGuard<'static, ()>,
}

pub(super) fn checkpoint_recovery_test_guard() -> CheckpointRecoveryTestGuard {
    static EXCLUSIVE_PROVIDER_COUNTER: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let exclusive = EXCLUSIVE_PROVIDER_COUNTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    CheckpointRecoveryTestGuard {
        _query_observation: primary_graph::WorthQueryExecutionObservation::install_on_this_thread(),
        _exclusive: exclusive,
    }
}
