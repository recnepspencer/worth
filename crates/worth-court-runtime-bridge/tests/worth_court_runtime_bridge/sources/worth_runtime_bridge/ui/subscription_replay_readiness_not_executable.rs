use worth_runtime_bridge::facade::BridgeSubscriptionDeliveryWindowReplayReadiness;

fn cannot_execute_replay(readiness: &BridgeSubscriptionDeliveryWindowReplayReadiness) {
    let _ = readiness.execute_replay();
}

fn main() {}
