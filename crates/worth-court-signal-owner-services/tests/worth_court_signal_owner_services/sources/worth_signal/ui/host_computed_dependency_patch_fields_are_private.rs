use worth_signal::facade::HostComputedDependencyPatch;
use worth_signal::facade::NodeId;

fn main() {
    let _ = HostComputedDependencyPatch {
        node: NodeId::new(1, 0),
        previous_dependencies: loop {},
        next_dependencies: loop {},
        added_dependencies: loop {},
        removed_dependencies: loop {},
        retained_dependency_count: 0,
    };
}
