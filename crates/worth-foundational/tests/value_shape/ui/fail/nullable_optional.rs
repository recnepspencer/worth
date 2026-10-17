use worth_foundational::facade::{CanonicalValue, WorthValue};

#[derive(WorthValue)]
struct NestedOptional {
    value: Option<Option<u64>>,
}

#[derive(WorthValue)]
struct UnitMarker;

#[derive(WorthValue)]
struct OptionalUnit {
    value: Option<UnitMarker>,
}

fn main() {
    let _ = NestedOptional::SHAPE_DEPTH;
    let _ = OptionalUnit::SHAPE_DEPTH;
}
