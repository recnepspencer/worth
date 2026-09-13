use super::super::{
    SignalBranchExecutionCell, SignalBranchRegistry, SignalOwnerLifecycleState,
    SignalOwnerServiceCounters,
};

#[test]
fn shared_signal_kernel_owners_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<SignalOwnerServiceCounters>();
    assert_send_sync::<SignalOwnerLifecycleState>();
    assert_send_sync::<SignalBranchRegistry<u64>>();
    assert_send_sync::<SignalBranchExecutionCell<u64>>();
}
