use worth_foundational::facade::WorthValue;

#[derive(WorthValue)]
#[serde(tag = "kind")]
enum TuplePayload {
    Unsupported(String),
}

#[derive(WorthValue)]
#[serde(tag = "kind")]
enum TagCollision {
    Unsupported { kind: String },
}
fn main() {}
