use worth_runtime_bridge::facade::BridgeAsyncDeniedCompletion;

fn cannot_construct_denied_async_completion_directly() {
    let _ = BridgeAsyncDeniedCompletion {};
}

fn main() {}
