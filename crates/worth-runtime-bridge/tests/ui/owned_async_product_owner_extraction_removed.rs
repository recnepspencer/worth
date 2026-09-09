use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

fn steal_product_issuer(bridge: &mut BridgeSealedRuntimeAssembly) {
    let _owner = bridge.take_owned_async_product_source_owner().unwrap();
}

fn main() {}
