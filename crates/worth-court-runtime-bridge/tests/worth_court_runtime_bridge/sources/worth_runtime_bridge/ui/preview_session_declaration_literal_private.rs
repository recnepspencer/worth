use worth_runtime_bridge::facade::BridgePreviewSessionDeclaration;

fn reconstruct(value: BridgePreviewSessionDeclaration) -> BridgePreviewSessionDeclaration {
    BridgePreviewSessionDeclaration { ..value }
}

fn main() {
    let _ = reconstruct;
}
