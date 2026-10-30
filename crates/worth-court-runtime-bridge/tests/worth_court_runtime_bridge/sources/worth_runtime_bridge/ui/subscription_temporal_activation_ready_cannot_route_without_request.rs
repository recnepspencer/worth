use worth_runtime_bridge::facade::{
    BridgeTemporalCauseRecord, BridgeTemporalSubscriptionActivationReady, RuntimeBridge,
};

fn fake<T>() -> T {
    panic!("type-only")
}

fn main() {
    let runtime: RuntimeBridge = fake();
    let ready: BridgeTemporalSubscriptionActivationReady = fake();
    let _ = runtime.route_temporal_wake(&ready, None::<&BridgeTemporalCauseRecord>);
}
