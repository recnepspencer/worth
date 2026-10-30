use worth_runtime_bridge::facade::{
    BridgeAsyncRequestAdmissionRequest, LoweredBridgeAsyncSourceDeclaration,
    ValidatedBridgeAsyncRequestBasisBinding,
};

fn fake<T>() -> T {
    panic!("type-only")
}

fn main() {
    let lowered: LoweredBridgeAsyncSourceDeclaration = fake();
    let binding: ValidatedBridgeAsyncRequestBasisBinding = fake();
    let _ = BridgeAsyncRequestAdmissionRequest::subscription_backed(&lowered, &binding);
}
