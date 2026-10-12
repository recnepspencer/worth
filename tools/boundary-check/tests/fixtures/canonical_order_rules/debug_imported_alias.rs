mod types {
    pub type Ops = std::collections::HashSet<std::any::TypeId>;
}
mod owner {
    use super::types::Ops;
    #[derive(Debug)]
    struct Inventory {
        operations: Ops,
    }
}
