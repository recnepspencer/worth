use worth_signal::facade::*;

fn main() {
    let graph = SignalGraph::new();
    let _runtime = SignalRuntime::builder(graph).build();
}
