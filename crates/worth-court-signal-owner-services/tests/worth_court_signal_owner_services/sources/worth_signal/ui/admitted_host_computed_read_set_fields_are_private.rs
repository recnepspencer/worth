use worth_signal::facade::AdmittedHostComputedReadSet;
use worth_signal::facade::NodeId;

fn main() {
    let _ = AdmittedHostComputedReadSet {
        node: NodeId::new(1, 0),
        dependencies: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
