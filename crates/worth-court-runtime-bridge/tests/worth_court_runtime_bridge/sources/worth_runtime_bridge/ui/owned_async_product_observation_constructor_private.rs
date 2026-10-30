use worth_runtime_bridge::facade::BridgeOwnedAsyncProductObservation;

fn fabricate<T>() -> T {
    loop {}
}

fn main() {
    let _forged = BridgeOwnedAsyncProductObservation {
        occurrence: std::sync::Arc::new(()),
        payload: fabricate(),
    };
}
