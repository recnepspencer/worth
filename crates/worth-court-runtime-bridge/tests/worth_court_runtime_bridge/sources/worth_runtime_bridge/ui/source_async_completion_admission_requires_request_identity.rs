use worth_runtime_bridge::facade::{
    BridgeAsyncCompletionRejection, RuntimeBridge, ValidatedBridgeAsyncCompletionEnvelope,
};

fn cannot_admit_completion_without_request_identity(
    runtime: &RuntimeBridge,
    envelope: &ValidatedBridgeAsyncCompletionEnvelope,
) -> Result<(), BridgeAsyncCompletionRejection> {
    runtime.admit_async_completion(envelope)
}

fn main() {
    let _ = cannot_admit_completion_without_request_identity;
}
