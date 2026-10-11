struct Inventory {
    operations: std::collections::HashSet<std::any::TypeId>,
}
impl std::fmt::Debug for Inventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inventory")
            .field("operations", &self.operations.len())
            .finish()
    }
}
#[derive(Debug)]
struct Values {
    names: std::collections::HashMap<String, std::any::TypeId>,
}
