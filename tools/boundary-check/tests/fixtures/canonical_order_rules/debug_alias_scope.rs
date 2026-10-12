mod first {
    type Ops = std::collections::HashSet<std::any::TypeId>;
    #[derive(Debug)]
    struct Inventory {
        operations: Ops,
    }
}
mod second {
    type Ops = Vec<u8>;
    #[derive(Debug)]
    struct Inventory {
        operations: Ops,
    }
}
