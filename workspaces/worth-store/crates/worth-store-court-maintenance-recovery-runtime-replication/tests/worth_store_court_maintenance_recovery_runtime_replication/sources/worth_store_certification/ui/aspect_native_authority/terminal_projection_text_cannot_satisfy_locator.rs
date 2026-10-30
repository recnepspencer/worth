use worth_store_aspect_native::StoreAspectBoundaryLocator;

fn require_store_locator(_locator: StoreAspectBoundaryLocator) {}

fn main() {
    let terminal_projection_text = "store.physical.segment.identity".to_owned();

    require_store_locator(terminal_projection_text);
}
