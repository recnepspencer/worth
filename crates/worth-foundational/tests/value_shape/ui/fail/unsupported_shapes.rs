use std::collections::HashMap;
use worth_foundational::facade::WorthValue;

#[derive(WorthValue)]
struct UnboundedMap {
    entries: HashMap<String, String>,
}

#[derive(WorthValue)]
struct DirectlyRecursive {
    children: Vec<DirectlyRecursive>,
}

fn main() {}
