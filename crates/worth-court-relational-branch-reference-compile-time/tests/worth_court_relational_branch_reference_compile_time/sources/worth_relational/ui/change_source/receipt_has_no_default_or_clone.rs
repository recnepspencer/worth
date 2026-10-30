use worth_relational::facade::change_source::RelationalChangeReceipt;

fn requires_default<T: Default>() {}
fn requires_clone<T: Clone>() {}

fn main() {
    requires_default::<RelationalChangeReceipt>();
    requires_clone::<RelationalChangeReceipt>();
}
