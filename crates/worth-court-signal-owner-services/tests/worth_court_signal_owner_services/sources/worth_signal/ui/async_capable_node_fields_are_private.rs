use worth_signal::facade::core::AsyncCapableNode;

fn fake() -> AsyncCapableNode {
    panic!("compile-fail fixture")
}

fn main() {
    let node = fake();
    let _ = node.node;
}
