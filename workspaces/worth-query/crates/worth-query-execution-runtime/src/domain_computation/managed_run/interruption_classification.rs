use worth_runtime_bridge::facade::{
    BridgeExecutionQueuePressureState, BridgeExecutionSafePointSignalState,
};

use super::{WorthQueryManagedRunTerminalKind, WorthQueryManagedSafePointObservation};

pub(super) fn producer_terminal_kind(
    observation: &WorthQueryManagedSafePointObservation,
) -> Option<WorthQueryManagedRunTerminalKind> {
    match observation.signal_state() {
        BridgeExecutionSafePointSignalState::Active => (observation.pressure_state()
            == BridgeExecutionQueuePressureState::Saturated)
            .then_some(WorthQueryManagedRunTerminalKind::Exhausted),
        state => signal_terminal_kind(state),
    }
}

pub(super) fn consumer_terminal_kind(
    observation: &WorthQueryManagedSafePointObservation,
) -> Option<WorthQueryManagedRunTerminalKind> {
    signal_terminal_kind(observation.signal_state())
}

fn signal_terminal_kind(
    state: BridgeExecutionSafePointSignalState,
) -> Option<WorthQueryManagedRunTerminalKind> {
    match state {
        BridgeExecutionSafePointSignalState::Active => None,
        BridgeExecutionSafePointSignalState::Cancelled => {
            Some(WorthQueryManagedRunTerminalKind::Cancelled)
        }
        BridgeExecutionSafePointSignalState::TimedOut => {
            Some(WorthQueryManagedRunTerminalKind::TimedOut)
        }
        BridgeExecutionSafePointSignalState::Fulfilled
        | BridgeExecutionSafePointSignalState::Rejected
        | BridgeExecutionSafePointSignalState::Superseded => {
            Some(WorthQueryManagedRunTerminalKind::Degraded)
        }
    }
}
