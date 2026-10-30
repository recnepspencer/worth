use worth_runtime_bridge::facade::{
    BridgeTemporalAsyncCertificationBundleDraft, RuntimeBridge,
};

fn fake<T>() -> T {
    panic!("fixture should never run")
}

fn main() {
    let runtime: RuntimeBridge = fake();
    let draft: BridgeTemporalAsyncCertificationBundleDraft = fake();
    let _ = runtime.export_temporal_async_certification_bundle(&draft);
}
