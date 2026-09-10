use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

fn inject_product_authority(bridge: &BridgeSealedRuntimeAssembly) {
    bridge
        .install_runtime_world_owned_async_product_authority(())
        .unwrap();
}

fn main() {}
