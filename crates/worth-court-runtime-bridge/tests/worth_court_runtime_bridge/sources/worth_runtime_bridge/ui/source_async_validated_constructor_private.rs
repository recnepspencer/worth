use worth_runtime_bridge::facade::ValidatedBridgeAsyncSourceDeclaration;

fn cannot_construct_validated_async_source_declaration_directly() {
    let _ = ValidatedBridgeAsyncSourceDeclaration {};
}

fn main() {}
