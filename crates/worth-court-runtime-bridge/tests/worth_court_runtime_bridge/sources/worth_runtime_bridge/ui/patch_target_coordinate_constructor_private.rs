use worth_runtime_bridge::facade::{
    BridgeCommittedPatchTarget, BridgePatchTargetCoordinate,
};

fn main() {
    let _coordinate = BridgePatchTargetCoordinate::new(
        "entity-1",
        sealed_native_patch_target_placeholder(),
    );
}

fn sealed_native_patch_target_placeholder() -> BridgeCommittedPatchTarget {
    loop {}
}
