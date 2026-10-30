use std::{collections::HashMap, sync::Arc};
use worth_execution::ChargedBytes;

#[derive(ChargedBytes)]
struct Unsupported {
    shared: Arc<Vec<String>>,
    mapping: HashMap<String, String>,
}
fn main() {}
