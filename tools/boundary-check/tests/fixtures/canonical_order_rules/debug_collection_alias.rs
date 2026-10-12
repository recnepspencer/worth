type Operations = std::collections::HashSet<std::any::TypeId>;
#[derive(Debug)]
struct Inventory {
    operations: Operations,
}
