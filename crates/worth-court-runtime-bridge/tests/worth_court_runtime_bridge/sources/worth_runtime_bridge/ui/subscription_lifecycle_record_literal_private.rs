use worth_runtime_bridge::facade::BridgeSubscriptionLifecycleRecord;

fn main() {
    let _ = BridgeSubscriptionLifecycleRecord {
        state_kind: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
