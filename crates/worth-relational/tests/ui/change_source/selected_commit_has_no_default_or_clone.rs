use worth_relational::facade::change_source::RelationalSelectedCommit;

fn requires_default<T: Default>() {}
fn requires_clone<T: Clone>() {}

fn main() {
    requires_default::<RelationalSelectedCommit>();
    requires_clone::<RelationalSelectedCommit>();
}
