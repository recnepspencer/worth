use worth_server::WorthServerLoweredOperationPlan;

fn main() {
    consume(impossible());
}

fn consume(plan: WorthServerLoweredOperationPlan) {
    let _ = plan.into_query_handoff();
}

fn impossible() -> WorthServerLoweredOperationPlan {
    loop {}
}
