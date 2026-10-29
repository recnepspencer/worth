use worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration;

fn cannot_construct_lowered_async_source_declaration_directly() {
    let _ = LoweredBridgeAsyncSourceDeclaration {};
}

fn main() {}
