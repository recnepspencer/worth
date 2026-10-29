pub(crate) use worth_query_testkit_certification_operation_body::facade::certification::native_operation_bodies::operation_body::optional_input::*;

#[test]
fn freshly_readmitted_optional_input_forms_require_optional_destinations() {
    reject_required_assignment();
    reject_required_initializer();
}
