use worth_foundational::AspectValue;
use worth_store_aspect_native::StoreAspectBoundaryFact;

fn require_store_boundary_fact(_fact: StoreAspectBoundaryFact) {}

fn main() {
    let raw_value = AspectValue::Null;

    require_store_boundary_fact(raw_value);
}
