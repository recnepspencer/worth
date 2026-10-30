use std::rc::Rc;

use worth_signal::facade::{SignalGraph, SignalRuntime};

fn invalid_local_only_context() {
    let mut runtime = SignalRuntime::build_for::<Rc<()>>(SignalGraph::new());
    let _ = runtime.owner_component_services();
}

fn valid_send_sync_context() {
    let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let _ = runtime.owner_component_services();
}

fn main() {}
