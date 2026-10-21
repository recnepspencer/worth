use std::collections::HashMap;
use worth_foundational::facade::WorthValue;

#[derive(WorthValue)]
struct UnboundedMap {
    entries: HashMap<String, String>,
}

#[derive(WorthValue)]
struct UnboundedShared {
    children: std::sync::Arc<String>,
}

fn main() {}
