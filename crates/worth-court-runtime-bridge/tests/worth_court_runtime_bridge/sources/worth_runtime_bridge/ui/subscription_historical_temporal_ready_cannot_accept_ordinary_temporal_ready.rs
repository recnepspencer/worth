use worth_runtime_bridge::facade::{
    BridgeHistoricalTemporalReadiness, BridgeTemporalSubscriptionActivationReady, RuntimeBridge,
};

fn fake<T>() -> T {
    panic!("type-only")
}

fn main() {
    let runtime: RuntimeBridge = fake();
    let ready: BridgeTemporalSubscriptionActivationReady = fake();
    let _: BridgeHistoricalTemporalReadiness =
        runtime.prepare_historical_temporal_readiness(&ready);
}
