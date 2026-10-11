#[derive(Debug)]
struct Direct {
    operations: std::collections::HashSet<std::any::TypeId>,
}
#[derive(Debug)]
struct Tuple {
    operations: std::collections::HashMap<(u8, std::any::TypeId), String>,
}
