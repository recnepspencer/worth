use worth_signal::facade::AdmittedHostComputedReadSet;
use worth_signal::facade::NodeId;

fn main() {
    let _ = AdmittedHostComputedReadSet {
        node: NodeId::new(1, 0),
        dependencies: loop {},
    };
}
