pub(crate) use worth_query_testkit_certification::facade::certification::runtime_world_branching::*;

#[test]
fn reconstruction_panic_restores_runtime_owners_for_retry() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world = CourtroomWorld::publish("ready");
    world.reconstruction_panic.set(true);

    assert!(world.reinstall_conditional_runtime().is_err());

    world.reconstruction_panic.set(false);
    assert!(world.reinstall_conditional_runtime().is_ok());
    let receipt = observe(request_execution, &world);
    assert_eq!(receipt.committed_operation_count(), 1);
}

#[test]
fn host_installs_and_executes_a_due_temporal_application_operation() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::host_installs_and_executes_due_operation(request_execution);
}

#[test]
fn public_product_journey_publishes_delivers_executes_and_cleans_up() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    public_product_journey::publishes_delivers_executes_and_cleans_up(request_execution);
}

#[test]
fn public_branch_facade_creates_and_selects_all_component_postures() {
    public_product_journey::creates_and_selects_all_component_postures();
}

#[test]
fn public_world_no_effect_retains_conditional_provenance() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    public_live_route::world_no_effect_retains_conditional_provenance(request_execution);
}

#[test]
fn public_live_query_receives_a_real_conditional_world_publication() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    public_live_route::live_query_receives_conditional_world_publication(request_execution);
}

#[test]
fn independent_products_advance_and_retain_exact_conditional_definitions() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    public_product_journey::independent_products_advance_and_retain_exact_definitions(
        request_execution,
    );
}

#[test]
fn application_commits_relational_and_signal_in_one_world_publication() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    public_product_journey::application_commits_relational_and_signal_in_one_world_publication(
        request_execution,
    );
}

#[test]
fn selected_sibling_mutation_carries_one_world_occurrence() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    public_product_journey::selected_sibling_mutation_carries_one_world_occurrence(
        request_execution,
    );
}

#[test]
fn shared_component_sibling_cannot_claim_another_product_commit() {
    public_product_journey::shared_component_sibling_cannot_claim_another_product_commit();
}

#[test]
fn shared_component_sibling_revalidates_its_own_security() {
    public_product_journey::shared_component_sibling_revalidates_its_own_security();
}

#[test]
fn temporal_wake_post_commit_snapshot_recovery_preserves_its_product_commit() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom_settlement::temporal_wake_post_commit_snapshot_recovery_preserves_its_product_commit(
        request_execution,
    );
}

#[test]
fn future_temporal_operation_waits_until_its_due_coordinate() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::future_temporal_operation_waits_until_due(request_execution);
}

#[test]
fn unrelated_rows_do_not_expand_conditional_observation_work() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::unrelated_rows_do_not_expand_conditional_observation_work(request_execution);
}

#[test]
fn cancellation_after_publication_retires_the_stale_wake_without_effects() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::cancellation_after_publication_retires_stale_wake(request_execution);
}

#[test]
fn active_successor_revision_replaces_the_predecessor_wake() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::active_successor_revision_replaces_predecessor_wake(request_execution);
}

#[test]
fn busy_branch_heals_a_lagging_cursor_inside_observation() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::busy_branch_heals_a_lagging_cursor_inside_observation(request_execution);
}

#[test]
fn native_writer_commit_reconsiders_a_suppressed_wake() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::native_writer_commit_reconsiders_a_suppressed_wake(request_execution);
}

#[test]
fn a_duplicate_clock_reading_after_a_foreign_commit_emits_its_invalidation() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::a_duplicate_clock_reading_after_a_foreign_commit_emits_its_invalidation(
        request_execution,
    );
}

#[test]
fn a_reordered_clock_reading_after_a_foreign_commit_keeps_its_invalidation() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::a_reordered_clock_reading_after_a_foreign_commit_keeps_its_invalidation_for_the_next_accepted_observation(request_execution);
}

#[test]
fn a_stale_clock_reading_after_a_foreign_commit_keeps_its_invalidation() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::a_stale_clock_reading_after_a_foreign_commit_keeps_its_invalidation_for_the_next_accepted_observation(request_execution);
}

#[test]
fn reinstallation_reconstructs_active_authoritative_work() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom_lifecycle::reinstallation_reconstructs_active_authoritative_work(request_execution);
}

#[test]
fn reinstallation_restores_no_cancelled_or_completed_work() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom_lifecycle::reinstallation_restores_no_terminal_work(request_execution);
}

#[test]
fn reinstallation_after_eligibility_retries_freshly() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom_lifecycle::reinstallation_after_eligibility_retries_freshly(request_execution);
}

#[test]
fn reinstallation_after_commit_cannot_duplicate_the_effect() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom_lifecycle::reinstallation_after_commit_cannot_duplicate_effect(request_execution);
}

#[test]
fn reinstallation_revokes_captured_granular_batches() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom_lifecycle::reinstallation_revokes_captured_granular_batches(request_execution);
}

#[test]
fn closing_the_runtime_releases_conditional_inventory_and_revokes_handles() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom_lifecycle::closing_runtime_releases_inventory_and_revokes_handles(request_execution);
}

#[test]
fn dropping_the_runtime_releases_exact_conditional_inventory() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom_lifecycle::dropping_runtime_releases_exact_inventory(request_execution);
}

#[test]
fn suppressed_wake_is_reconsidered_after_authoritative_truth_changes() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::suppressed_wake_is_reconsidered_after_truth_change(request_execution);
}

#[test]
fn precondition_panic_isolated_and_retry_succeeds() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::precondition_panic_isolated_and_retry_succeeds(request_execution);
}

#[test]
fn predicate_panic_does_not_corrupt_runtime_owners() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::predicate_panic_does_not_corrupt_runtime_owners(request_execution);
}

#[test]
fn duplicate_reordered_and_foreign_clocks_fail_closed() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::duplicate_reordered_and_foreign_clocks_fail_closed(request_execution);
}

#[test]
fn provider_replacement_requires_fresh_runtime_publication() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::provider_replacement_requires_fresh_runtime_publication(request_execution);
}

pub(crate) mod application_readiness_reports_current_query_basis_without_leakin;

pub(crate) mod conditional_clock_observation_preserves_real_snapshot_capacity_d;

pub(crate) mod cumulative;

pub(crate) mod selected_product_lifecycle;

pub(crate) mod selected_product_reads;

pub(crate) mod shared_root_progress;

pub(crate) mod successor_generation_requires_fresh_typed_rebinding;
