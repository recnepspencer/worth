use worth_runtime_bridge::facade::AdmittedBridgeAsyncCompletion;

fn cannot_construct_admitted_async_completion_directly() {
    let _ = AdmittedBridgeAsyncCompletion {};
}

fn main() {}
