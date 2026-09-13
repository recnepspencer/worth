use worth_signal::facade::branch::SignalOwnerOperationBoundary;
use worth_signal::facade::{SignalGraph, SignalRuntime};

fn invalid_default_control_surface() {
    let _ = SignalOwnerOperationBoundary::BeforeCanonicalMovement;
}

fn valid_default_surface() {
    let _ = worth_signal::facade::branch::SignalOwnerLifecycleObservation::Closed;
}

fn invalid_default_accessor() {
    let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let _ = runtime.owner_operation_control();
}

fn main() {}
