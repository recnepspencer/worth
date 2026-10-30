use worth_runtime_bridge::facade::{
    BridgeSubscriptionBundleField, BridgeSubscriptionBundleFieldState,
};

fn main() {
    let _field = BridgeSubscriptionBundleField {
        field_name: "comparison_inputs".into(),
        field_state: BridgeSubscriptionBundleFieldState::Present,
        field_digest: sealed_authority_placeholder(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
