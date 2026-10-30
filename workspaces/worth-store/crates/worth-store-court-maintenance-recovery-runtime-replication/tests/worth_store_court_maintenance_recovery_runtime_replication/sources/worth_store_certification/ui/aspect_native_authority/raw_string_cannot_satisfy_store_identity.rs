use worth_store_aspect_native::StoreAspectIdentity;

fn require_store_identity(_identity: StoreAspectIdentity) {}

fn main() {
    let raw_identity = String::from("store.physical.segment.identity");

    require_store_identity(raw_identity);
}
