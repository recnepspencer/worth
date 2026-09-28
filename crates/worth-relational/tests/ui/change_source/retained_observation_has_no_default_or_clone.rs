use worth_relational::facade::change_source::RelationalRetainedObservation;

fn requires_default<T: Default>() {}
fn requires_clone<T: Clone>() {}

fn main() {
    requires_default::<RelationalRetainedObservation>();
    requires_clone::<RelationalRetainedObservation>();
}
