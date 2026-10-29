use worth_runtime_bridge::facade::AdmittedBridgeAsyncWriteback;

fn cannot_construct_admitted_async_writeback_directly() {
    let _ = AdmittedBridgeAsyncWriteback {};
}

fn main() {}
