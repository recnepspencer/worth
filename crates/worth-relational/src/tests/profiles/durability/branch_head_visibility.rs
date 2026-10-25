use crate::facade::history::BranchId;
use crate::tests::support::*;

#[test]
fn branch_head_visibility_updates_incrementally_under_branch_churn() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .visibility_cache_policy(VisibilityCachePolicy {
            enabled: true,
            protect_branch_heads: true,
            protect_replay_retained: false,
            protect_active_snapshots: false,
            recent_version_window: 0,
        })
        .build();

    let base = create_entity_outcome(execution, &runtime, "base");
    let entity = changed_entities(&base)[0];
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("analysis".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();

    runtime.performance_access().reset_counters();
    for revision in 0..3 {
        let _ = update_entity(execution, &runtime, entity, &format!("base-r{revision}"));
    }

    let stats = runtime.storage_access().storage_stats();
    let counters = runtime.performance_access().counters();

    assert_eq!(stats.protected_visibility_version_count, 2);
    assert_eq!(stats.cached_visibility_version_count, 0);
    assert_eq!(stats.recent_visibility_cache_count, 0);
    assert_eq!(counters.visibility_cache_branch_head_promotions, 3);
}

#[test]
fn branch_head_protection_can_be_lazy_without_populating_visibility_cache() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .visibility_cache_policy(VisibilityCachePolicy {
            enabled: true,
            protect_branch_heads: true,
            protect_replay_retained: false,
            protect_active_snapshots: false,
            recent_version_window: 0,
        })
        .build();

    let base = create_entity_outcome(execution, &runtime, "base");
    let entity = changed_entities(&base)[0];
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("analysis".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let _ = update_entity(execution, &runtime, entity, "base-updated");

    let stats = runtime.storage_access().storage_stats();

    assert_eq!(stats.protected_visibility_version_count, 2);
    assert_eq!(stats.cached_visibility_version_count, 0);
    assert_eq!(stats.recent_visibility_cache_count, 0);

    let _ = runtime
        .read_truth()
        .read_version(execution, base.version_id)
        .expect("read request remains live");
    let warmed_stats = runtime.storage_access().storage_stats();
    assert_eq!(warmed_stats.cached_visibility_version_count, 0);
    assert_eq!(warmed_stats.protected_visibility_version_count, 2);
}
