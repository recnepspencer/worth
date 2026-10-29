use worth_store_aspect_native::{StoreAspectIdentity, StoreTerminalJsonProjection};

fn main() {
    let identity: StoreAspectIdentity = todo!();
    let _ = StoreTerminalJsonProjection::from_terminal_projection_document(
        identity,
        "segment-0011".into(),
    );
}
