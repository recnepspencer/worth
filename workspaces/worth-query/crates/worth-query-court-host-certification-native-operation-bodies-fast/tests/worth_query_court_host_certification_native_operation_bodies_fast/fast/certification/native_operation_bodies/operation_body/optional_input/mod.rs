pub(crate) use worth_query_worlds::facade::certification::native_operation_bodies::operation_body::optional_input::*;

#[test]
fn freshly_readmitted_optional_input_forms_require_optional_destinations() {
    reject_required_assignment();
    reject_required_initializer();
}
