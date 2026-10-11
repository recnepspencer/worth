//! Owned observation scope shared by real checkpoint recovery probes.
use super::primary_graph;

pub(super) struct CheckpointRecoveryTestGuard {
    _runtime_oracle_factory: primary_graph::WorthQueryInvalidationEquivalenceForTest,
    _query_observation: primary_graph::WorthQueryExecutionObservation,
    _exclusive: std::sync::MutexGuard<'static, ()>,
}

pub(super) fn checkpoint_recovery_test_guard() -> CheckpointRecoveryTestGuard {
    static EXCLUSIVE_PROVIDER_COUNTER: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let exclusive = EXCLUSIVE_PROVIDER_COUNTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // Law 2, scope: construction arms each fixture runtime's own receiver.
    // Its workers read the receiver arm; the factory guard clears only its arms.
    CheckpointRecoveryTestGuard {
        _runtime_oracle_factory:
            primary_graph::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test(),
        _query_observation: primary_graph::WorthQueryExecutionObservation::install_on_this_thread(),
        _exclusive: exclusive,
    }
}
