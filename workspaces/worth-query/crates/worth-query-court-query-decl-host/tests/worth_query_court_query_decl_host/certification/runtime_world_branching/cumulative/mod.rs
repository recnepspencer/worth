pub(crate) use worth_query_testkit_certification::facade::certification::runtime_world_branching::cumulative::*;

#[test]
fn primary_runtime_carries_owner_evidence() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    runtime_composition::assert_primary_runtime_composition(request_execution);
}

#[test]
fn duplicate_and_reordered_delivery_converges() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    delivery_convergence::assert_duplicate_and_reordered_convergence(request_execution);
}

#[test]
fn foreign_primary_source_is_denied() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    query_runtime_world::assert_foreign_primary_source_is_denied_at_build(request_execution);
}

#[test]
fn admitted_read_retains_its_basis() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    runtime_composition::assert_head_advance_preserves_admitted_granular_read(request_execution);
}

#[test]
fn granular_receipt_uses_execution_basis() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    runtime_composition::assert_granular_receipt_uses_execution_snapshot_basis(request_execution);
}

#[test]
fn financial_runtime_delivers_exact_patches() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    financial_runtime_world::assert_financial_host_curve_delivery(request_execution);
    financial_runtime_world::assert_financial_curve_query_patch(request_execution);
    financial_runtime_world::assert_sibling_curve_record_does_no_query_work(request_execution);
    financial_runtime_world::assert_suppressed_quote_has_no_query_patch(request_execution);
    financial_runtime_world::assert_ordered_portfolio_membership(request_execution);
    financial_runtime_world::assert_shared_financial_execution_and_publication(request_execution);
    financial_runtime_world::assert_shared_financial_disclosure_revalidation(request_execution);
}

#[test]
fn reconstruction_requires_current_owner_correspondence() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    shared_lifecycle::assert_correspondence_rebind_restore(request_execution);
}

#[test]
fn sibling_writer_commits_while_combined_publication_is_parked() {
    cost::assert_sibling_writer_progress_while_combined_is_parked();
}

#[test]
fn graph_work_capacity_obeys_installed_exact_bound() {
    cost::assert_graph_work_capacity_bounds();
}
