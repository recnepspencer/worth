use worth_foundational::StructAspectValue;

fn reconstruct(value: StructAspectValue) -> StructAspectValue {
    StructAspectValue { ..value }
}

fn main() {
    let _ = reconstruct;
}
