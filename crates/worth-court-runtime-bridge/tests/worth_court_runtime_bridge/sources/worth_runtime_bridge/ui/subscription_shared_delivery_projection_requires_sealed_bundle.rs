use worth_runtime_bridge::facade::{
    BridgeSharedConsumerDeliveryBundleDraft, RuntimeBridge,
};

fn fake<T>() -> T {
    panic!("fixture should never run")
}

fn main() {
    let runtime: RuntimeBridge = fake();
    let draft: BridgeSharedConsumerDeliveryBundleDraft = fake();
    let _ = runtime.project_shared_delivery_consumer(&draft, 0);
}
