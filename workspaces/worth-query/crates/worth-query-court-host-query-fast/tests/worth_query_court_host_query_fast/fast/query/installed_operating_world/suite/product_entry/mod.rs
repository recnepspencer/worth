pub(crate) use worth_query_worlds::facade::query::suite::product_entry::*;

#[test]
fn installed_product_entry_requires_typed_selection_and_retains_its_observation() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = workspace(request_execution, "typed-product-entry");
    let branch = workspace.current_world();
    let default = workspace.observe_operating_world(branch).unwrap();
    let held = default.product_branch();
    let expected = held.selected_commit().clone();
    let selected = workspace.observe_operating_world(branch).unwrap();
    assert!(held.has_same_selected_occurrence(selected.product_branch()));
    assert_eq!(default.product_branch().selected_commit(), &expected);
}

#[test]
fn product_selection_from_another_runtime_fails_before_binding() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first = workspace(request_execution, "first-product-entry");
    let second = workspace(request_execution, "second-product-entry");
    let branch = first.current_world();
    let denied = second
        .observe_operating_world(branch)
        .err()
        .expect("the runtime cannot adopt a foreign World selection");
    assert_eq!(denied.kind(), installed::WorthQueryOperatingWorldEntryDenialKind::Product(installed::WorthQueryOperatingWorldProductDenial::Admission(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial::Refused(worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionRefusalKind::ForeignOwner),
    )));
}
