use worth_foundational::{aspects, AspectValue};

fn main() {
    let _ = aspects()
        .patch()
        .whole_aspect()
        .set(AspectValue::Int64(1))
        .finish();
}
