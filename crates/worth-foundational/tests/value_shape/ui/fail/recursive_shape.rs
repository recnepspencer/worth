use worth_foundational::facade::{CanonicalValue, WorthValue};

#[derive(WorthValue)]
struct Left {
    right: Option<Right>,
}

#[derive(WorthValue)]
struct Right {
    left: Vec<Left>,
}

fn main() {
    let _ = Left::SHAPE_DEPTH;
}
