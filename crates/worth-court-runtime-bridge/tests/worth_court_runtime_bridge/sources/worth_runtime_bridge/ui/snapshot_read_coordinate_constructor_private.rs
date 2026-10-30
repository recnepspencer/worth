use worth_foundational::facade::AspectKey;
use worth_runtime_bridge::facade::{
    BridgeSnapshotReadCoordinate, SnapshotReadCorrelationId, SubscriptionSliceKind,
};

fn main() {
    let _coordinate = BridgeSnapshotReadCoordinate::new_subscription_slice(
        sealed_authority_placeholder::<SnapshotReadCorrelationId>(),
        "entity-1",
        AspectKey::new("profile").expect("valid native aspect key"),
        "forged-target-basis",
        SubscriptionSliceKind::SignalField,
    );
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
