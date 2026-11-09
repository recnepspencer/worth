use worth_foundational::facade::AspectKey;
use worth_runtime_bridge::facade::{
    BridgeSnapshotReadCoordinate, SnapshotReadCorrelationId, SnapshotReadTargetIdentity,
    SubscriptionSliceKind,
};

fn construct(correlation: SnapshotReadCorrelationId, target: SnapshotReadTargetIdentity) {
    let _coordinate = BridgeSnapshotReadCoordinate::new_subscription_slice(
        correlation,
        "entity-1",
        AspectKey::new("profile").expect("valid native aspect key"),
        target,
        SubscriptionSliceKind::SignalField,
    );
}

fn main() {}
