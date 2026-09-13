use std::time::Duration;

use worth_signal::facade::branch::{
    SignalOwnerOperationBoundary, SignalOwnerOperationControl, SignalOwnerOperationPause,
};
use worth_signal::facade::{SignalGraph, SignalRuntime};

fn assert_send_sync_clone<T: Send + Sync + Clone>(_: &T) {}
fn assert_send_sync<T: Send + Sync>(_: &T) {}

fn main() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(compile_operation_control_matrix)
        .expect("the operation-control fixture thread starts")
        .join()
        .expect("the operation-control fixture completes");
}

fn compile_operation_control_matrix() {
    let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let _services = runtime
        .owner_component_services()
        .expect("the concrete owner seals before issuing controls");
    let control: SignalOwnerOperationControl = runtime
        .owner_operation_control()
        .expect("the sealed owner issues the feature-gated control");
    assert_send_sync_clone(&control);
    let pause: SignalOwnerOperationPause =
        control.arm_pause_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
    assert_send_sync(&pause);
    let _ = pause.wait_until_reached(Duration::ZERO);
    pause.release();
    control.inject_panic_once(SignalOwnerOperationBoundary::OutcomeConstruction);
}
