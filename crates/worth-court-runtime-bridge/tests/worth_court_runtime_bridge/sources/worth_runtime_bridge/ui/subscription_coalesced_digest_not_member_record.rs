use worth_runtime_bridge::facade::{
    BridgeSubscriptionDeliveryMemberRecord, BridgeSubscriptionDeliveryWindowSealed,
};

fn cannot_substitute_window_digest(window: &BridgeSubscriptionDeliveryWindowSealed) {
    let _: &BridgeSubscriptionDeliveryMemberRecord = window.digest();
}

fn main() {}
