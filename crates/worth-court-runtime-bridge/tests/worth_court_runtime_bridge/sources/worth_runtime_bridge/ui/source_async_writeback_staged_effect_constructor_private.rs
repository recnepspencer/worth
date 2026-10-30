use worth_runtime_bridge::facade::StagedBridgeAsyncWritebackEffect;

fn cannot_construct_staged_async_writeback_effect_directly() {
    let _ = StagedBridgeAsyncWritebackEffect {};
}

fn main() {}
