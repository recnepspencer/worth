use super::*;

#[test]
fn query_context_capability_binds_branch_and_diff_contexts_without_mode_flags() {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let contexts = facade
        .query_context_capability()
        .expect("runtime-backed facade should admit query context capability");
    let left_preflight = execution_preflights::direct_runtime_preflight();
    let right_preflight = execution_preflights::alternate_basis_runtime_preflight();
    let left = contexts
        .capability()
        .admit_basis_context(
            basis_lifecycle().current_head(),
            QueryContextBindingSource::RuntimeCurrent(&left_preflight),
        )
        .expect("current branch context should admit");
    let right = contexts
        .capability()
        .admit_basis_context(
            basis_lifecycle().branch_head("branch:snapshot-2", true),
            QueryContextBindingSource::RuntimeBranch(&right_preflight),
        )
        .expect("branch context should admit");
    let diff = contexts
        .capability()
        .bind_diff_context(&left, &right)
        .expect("diff context should bind");
    let left_execution = contexts
        .capability()
        .execute_basis_context(&left)
        .expect("current context should execute through query-context capability");
    let right_execution = contexts
        .capability()
        .execute_basis_context(&right)
        .expect("branch context should execute through query-context capability");
    let basis_bundle = contexts
        .capability()
        .execute_basis_result_bundle(&left)
        .expect("basis result bundle should remain query-owned");
    let diff_bundle = contexts
        .capability()
        .shape_diff_result_bundle(&diff, &left_execution, &right_execution)
        .expect("diff result bundle should remain query-owned");
    let change_set = contexts
        .capability()
        .shape_diff_change_set(&diff, &left_execution, &right_execution)
        .expect("diff change-set should remain query-shaped");

    assert_eq!(left.family().as_str(), "current_branch_head");
    assert_eq!(right.family().as_str(), "branch_head");
    assert_eq!(diff.family().as_str(), "branch_to_branch");
    assert_eq!(
        basis_bundle.context().family().as_str(),
        "current_branch_head"
    );
    assert_eq!(
        basis_bundle.metadata().basis_digest(),
        basis_bundle.context().basis_digest()
    );
    assert_eq!(
        basis_bundle.metadata().result_digest(),
        basis_bundle.execution().result_digest()
    );
    assert!(!basis_bundle.replay_digest().is_empty());
    assert_eq!(change_set.comparison_basis_family(), diff.family());
    assert_eq!(
        diff_bundle.metadata().comparison_basis_family(),
        diff.family()
    );
    assert_eq!(
        diff_bundle.metadata().comparison_result_digest(),
        diff_bundle.change_set().result_digest()
    );
    assert_eq!(
        diff_bundle.metadata().prediction_drift_outcome(),
        diff_bundle.change_set().prediction_drift_outcome()
    );
    assert!(!diff_bundle.replay_digest().is_empty());
    assert!(!change_set.rows().is_empty());
    assert_eq!(contexts.counters().capability_lookup_count(), 1);
}

#[test]
fn support_report_includes_query_context_capability() {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");

    assert_eq!(
        support
            .descriptor(WorthQueryCapabilityFamily::QueryContext)
            .expect("query context descriptor should exist")
            .status(),
        WorthQueryCapabilityStatus::Admitted
    );
    assert!(report
        .admitted_capability_families()
        .contains(&WorthQueryCapabilityFamily::QueryContext));
    let profile = report
        .query_context_support_profile()
        .expect("query context support profile should be present");
    assert_eq!(
        profile
            .admitted_basis_families()
            .iter()
            .map(|family| family.as_str())
            .collect::<Vec<_>>(),
        vec![
            "current_branch_head",
            "branch_head",
            "historical_snapshot",
            "historical_commit",
            "preview_derived_historical"
        ]
    );
    assert_eq!(
        profile
            .admitted_comparison_families()
            .iter()
            .map(|family| family.as_str())
            .collect::<Vec<_>>(),
        vec![
            "branch_to_branch",
            "current_to_historical",
            "historical_to_historical",
            "preview_to_authoritative"
        ]
    );
    assert_eq!(
        profile
            .deferred_scope_markers()
            .iter()
            .map(|marker| marker.as_str())
            .collect::<Vec<_>>(),
        vec![
            "store_backed_historical",
            "store_backed_diff",
            "broad_collection_diff"
        ]
    );
}
