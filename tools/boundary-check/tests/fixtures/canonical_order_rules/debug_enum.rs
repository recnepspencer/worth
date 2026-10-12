#[derive(Debug)]
enum Inventory {
    Operations(std::collections::HashSet<std::any::TypeId>),
}
