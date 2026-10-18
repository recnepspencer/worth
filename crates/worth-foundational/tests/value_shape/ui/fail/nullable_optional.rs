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
    let _ = NestedOptional::value_shape();
    let _ = OptionalUnit::value_shape();
}
