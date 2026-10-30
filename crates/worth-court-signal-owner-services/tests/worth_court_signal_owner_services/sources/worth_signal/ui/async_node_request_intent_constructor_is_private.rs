use worth_signal::facade::{AsyncNodeRequestIntent, NodeId};

fn main() {
    let _ = AsyncNodeRequestIntent::new(NodeId::new(1, 0));
}
