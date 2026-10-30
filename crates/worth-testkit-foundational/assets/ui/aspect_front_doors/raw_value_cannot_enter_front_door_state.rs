use worth_foundational::{aspects, AspectValue};

fn main() {
    let _ = aspects().authoritative_state().admit([AspectValue::Int64(1)]);
}
