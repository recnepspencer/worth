use worth_signal::facade::{AsyncNodeRevalidationIntent, NodeId};

fn main() {
    let _ = AsyncNodeRevalidationIntent::new(NodeId::new(1, 0));
}
