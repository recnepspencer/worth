use worth_signal::facade::HostComputedDependencyPatch;
use worth_signal::facade::NodeId;

fn main() {
    let _ = HostComputedDependencyPatch {
        node: NodeId::new(1, 0),
        previous_dependencies: supplied(),
        next_dependencies: supplied(),
        added_dependencies: supplied(),
        removed_dependencies: supplied(),
        retained_dependency_count: 0,
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
