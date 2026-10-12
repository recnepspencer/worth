#[derive(Debug)]
struct Inventory {
    operations: std::collections::HashMap<[std::any::TypeId; 2], u8>,
}
