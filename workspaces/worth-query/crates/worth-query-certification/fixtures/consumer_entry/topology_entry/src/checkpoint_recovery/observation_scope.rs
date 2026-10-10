//! Owned observation scope shared by real checkpoint recovery probes.
use super::primary_graph;

pub(super) struct CheckpointRecoveryTestGuard {
    _exclusive: std::sync::MutexGuard<'static, ()>,
    _signal_observation: primary_graph::SignalRequestWorkObservation,
    _world_observation: primary_graph::RuntimeWorldReadObservation,
}

pub(super) fn checkpoint_recovery_test_guard() -> CheckpointRecoveryTestGuard {
    static EXCLUSIVE_PROVIDER_COUNTER: std::sync::Mutex<()> = std::sync::Mutex::new(());
    CheckpointRecoveryTestGuard {
        _exclusive: EXCLUSIVE_PROVIDER_COUNTER
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        _signal_observation: primary_graph::SignalRequestWorkObservation::install_on_this_thread(),
        _world_observation: primary_graph::RuntimeWorldReadObservation::install_on_this_thread(),
    }
}
