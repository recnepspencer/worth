use worth_foundational::{aspects, AspectValue};
use worth_store_aspect_native::StoreAspectAuthorityInput;

fn require_store_authority_input(_authority_input: StoreAspectAuthorityInput) {}

fn main() {
    let raw_struct = aspects()
        .vocabulary()
        .struct_value()
        .with_field("segment", AspectValue::Null)
        .finish()
        .unwrap();

    require_store_authority_input(raw_struct);
}
