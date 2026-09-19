use super::{
    cost, delivery_convergence, financial_runtime_world, model, query_runtime_world,
    runtime_composition, shared_lifecycle, structural_slopes,
};

#[test]
fn structural_slopes_cross_the_real_composition_root() {
    structural_slopes::assert_measured_bridge_and_result_slopes();
}

#[test]
fn primary_runtime_carries_owner_evidence() {
    runtime_composition::assert_primary_runtime_composition();
}

#[test]
fn duplicate_and_reordered_delivery_converges() {
    delivery_convergence::assert_duplicate_and_reordered_convergence();
}

#[test]
fn foreign_primary_source_is_denied() {
    query_runtime_world::assert_foreign_primary_source_is_denied_at_build();
}

#[test]
fn admitted_read_retains_its_basis() {
    runtime_composition::assert_head_advance_preserves_admitted_granular_read();
}

#[test]
fn granular_receipt_uses_execution_basis() {
    runtime_composition::assert_granular_receipt_uses_execution_snapshot_basis();
}

#[test]
fn financial_runtime_delivers_exact_patches() {
    financial_runtime_world::assert_financial_host_curve_delivery();
    financial_runtime_world::assert_financial_curve_query_patch();
    financial_runtime_world::assert_sibling_curve_record_does_no_query_work();
    financial_runtime_world::assert_suppressed_quote_has_no_query_patch();
    financial_runtime_world::assert_ordered_portfolio_membership();
    financial_runtime_world::assert_shared_financial_execution_and_publication();
    financial_runtime_world::assert_shared_financial_disclosure_revalidation();
}

#[test]
fn reconstruction_requires_current_owner_correspondence() {
    shared_lifecycle::assert_correspondence_rebind_restore();
}

#[test]
fn seeded_model_checks_public_branch_action_roster() {
    model::assert_seeded_public_action_roster();
}

#[test]
#[ignore = "scheduled broader seeded model; ordinary CI runs the bounded action roster"]
fn scheduled_seeded_model_broadens_public_action_roster() {
    model::assert_scheduled_public_action_roster();
}

#[test]
fn public_query_work_is_fixed_across_retained_populations() {
    cost::assert_public_population_slopes();
}

#[test]
fn public_query_work_is_fixed_across_independent_writers() {
    cost::assert_independent_writer_slopes();
}

#[test]
fn sibling_writer_commits_while_combined_publication_is_parked() {
    cost::assert_sibling_writer_progress_while_combined_is_parked();
}

#[test]
fn live_population_does_not_expand_publication_work() {
    cost::assert_live_publication_slopes();
}

#[test]
fn graph_work_capacity_obeys_installed_exact_bound() {
    cost::assert_graph_work_capacity_bounds();
}

#[test]
fn mutation_work_scales_only_with_touched_records() {
    cost::assert_touched_record_slopes();
}

#[test]
fn active_world_attempt_population_does_not_expand_query_work() {
    cost::assert_active_attempt_slopes();
}

#[test]
fn retained_product_partial_population_does_not_expand_query_work() {
    cost::assert_retained_partial_slopes();
}

#[test]
#[ignore = "scheduled timing evidence; structural counters are the ordinary regression gate"]
fn scheduled_public_read_timings_report_cold_and_warm_distributions() {
    cost::run_scheduled_public_read_timings();
}
