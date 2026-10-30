use worth_store_aspect_native::{StoreAspectIdentity, StoreTerminalProjectionText};

fn require_identity(_: StoreAspectIdentity) {}

fn main() {
    let text = StoreTerminalProjectionText::new_terminal_projection_text(
        "store.physical.segment.identity",
    );
    require_identity(text);
}
