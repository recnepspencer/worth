use worth_runtime_bridge::facade::{
    BridgeRetainedSubscriptionResumeBasis, RuntimeBridge,
};

fn fake<T>() -> T {
    panic!("private")
}

fn main() {
    let runtime = fake::<RuntimeBridge>();
    let retained = fake::<BridgeRetainedSubscriptionResumeBasis>();
    let _ = runtime.prepare_subscription_replay_readiness(&retained);
}
