#[derive(Debug)]
struct Inventory<K> {
    operations: std::collections::HashSet<K>,
}
