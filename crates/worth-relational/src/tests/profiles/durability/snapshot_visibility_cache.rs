use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::tests::support::*;

#[test]
fn visibility_cache_zero_window_does_not_accumulate_unprotected_history() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .visibility_cache_policy(VisibilityCachePolicy {
            enabled: true,
            protect_branch_heads: false,
            protect_replay_retained: false,
            protect_active_snapshots: true,
            recent_version_window: 0,
        })
        .build();

    let first = create_entity_outcome(execution, &runtime, "first");
    let second = create_entity_outcome(execution, &runtime, "second");

    runtime.performance_access().reset_counters();
    let _ = runtime
        .read_truth()
        .read_version(execution, first.version_id)
        .expect("read request remains live");
    let _ = runtime
        .read_truth()
        .read_version(execution, first.version_id)
        .expect("read request remains live");
    let stats = runtime.storage_access().storage_stats(execution).unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(stats.recent_visibility_cache_count, 0);
    assert_eq!(stats.cached_visibility_version_count, 0);
    assert_eq!(stats.protected_visibility_version_count, 0);
    assert_eq!(counters.visibility_cache_hits, 0);
    assert!(counters.visibility_cache_miss_reconstructions >= 2);
    assert_eq!(second.version_id.0, 2);
}

#[test]
fn explicit_snapshots_can_skip_cache_protection_and_still_read_until_release() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .visibility_cache_policy(VisibilityCachePolicy {
            enabled: true,
            protect_branch_heads: false,
            protect_replay_retained: false,
            protect_active_snapshots: false,
            recent_version_window: 0,
        })
        .build();

    let first = create_entity_outcome(execution, &runtime, "first");
    let snapshot = runtime.visibility_authority().snapshot();
    let entity = changed_entities(&first)[0];
    let _updated = update_entity(execution, &runtime, entity, "first-updated");

    let read_path = runtime
        .read_truth()
        .inspect_snapshot_read_path(&snapshot)
        .unwrap();
    assert!(read_path
        .entries
        .iter()
        .any(|entry| entry.code == DiagnosticCode::VisibilityCacheTransientRead));
    assert!(read_path.entries.iter().any(|entry| {
        entry.code == DiagnosticCode::SnapshotReadPathInspected
            && diagnostic_field(entry, "recent_candidate")
                == &RelationalDiagnosticValue::Bool(false)
    }));

    let read = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("read request remains live")
        .unwrap();
    let inspection = runtime.read_truth().inspect_snapshot(&snapshot).unwrap();
    let stats = runtime.storage_access().storage_stats(execution).unwrap();

    assert_eq!(
        read_entity_field(read.get_entity(entity).unwrap(), field_key("name")),
        Some("first".into())
    );
    assert_eq!(inspection.pinned_entity_count, 0);
    assert_eq!(stats.snapshot_count, 1);
    assert_eq!(stats.cached_visibility_version_count, 0);
    assert_eq!(stats.protected_visibility_version_count, 0);

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&snapshot)
        .is_ok());
    assert!(runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("read request remains live")
        .is_none());
}

#[test]
fn unprotected_active_snapshots_can_use_recent_cache_when_enabled() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .visibility_cache_policy(VisibilityCachePolicy {
            enabled: true,
            protect_branch_heads: false,
            protect_replay_retained: false,
            protect_active_snapshots: false,
            recent_version_window: 1,
        })
        .build();

    let first = create_entity_outcome(execution, &runtime, "first");
    let snapshot = runtime.visibility_authority().snapshot();
    let entity = changed_entities(&first)[0];
    let _updated = update_entity(execution, &runtime, entity, "first-updated");

    runtime.performance_access().reset_counters();
    let _ = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("read request remains live")
        .unwrap();
    let _ = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("read request remains live")
        .unwrap();

    let stats = runtime.storage_access().storage_stats(execution).unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(stats.snapshot_count, 1);
    assert_eq!(stats.protected_visibility_version_count, 0);
    assert_eq!(stats.recent_visibility_cache_count, 1);
    assert_eq!(stats.cached_visibility_version_count, 1);
    assert!(counters.visibility_exact_state_materializations >= 1);
    assert_eq!(counters.visibility_cache_miss_reconstructions, 0);
    assert!(counters.visibility_cache_hits >= 1);
}

#[test]
fn visibility_cache_recent_window_is_bounded_and_reports_hits() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .visibility_cache_policy(VisibilityCachePolicy {
            enabled: true,
            protect_branch_heads: true,
            protect_replay_retained: true,
            protect_active_snapshots: true,
            recent_version_window: 1,
        })
        .build();

    let first = create_entity_outcome(execution, &runtime, "first");
    let second = create_entity_outcome(execution, &runtime, "second");
    let third = create_entity_outcome(execution, &runtime, "third");

    assert_recent_version_admission_candidate(&runtime, first.version_id);

    runtime.performance_access().reset_counters();
    let _ = runtime
        .read_truth()
        .read_version(execution, first.version_id)
        .expect("read request remains live");
    let _ = runtime
        .read_truth()
        .read_version(execution, second.version_id)
        .expect("read request remains live");
    let _ = runtime
        .read_truth()
        .read_version(execution, second.version_id)
        .expect("read request remains live");
    let stats = runtime.storage_access().storage_stats(execution).unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(stats.recent_visibility_cache_count, 1);
    assert_eq!(stats.protected_visibility_version_count, 1);
    assert!(stats.cached_visibility_version_count <= 2);
    assert!(counters.visibility_cache_miss_reconstructions >= 2);
    assert!(counters.visibility_cache_hits >= 1);
    assert!(counters.visibility_cache_recent_evictions >= 1);
    assert_version_read_path_has_cache_hit(&runtime, second.version_id);
    assert_evicted_version_is_not_cached(&runtime, first.version_id);
    assert_eq!(
        third.version_id,
        runtime.history().latest_commit().unwrap().version_id
    );
}

#[test]
fn heavy_profiles_keep_recent_visibility_cache_small_under_sustained_history_reads() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::ChipSimulation)
        .schema_registry(test_schema_registry())
        .build();
    let mut versions = Vec::new();
    for index in 0..6 {
        versions.push(create_entity_outcome(execution, &runtime, &format!("e{index}")).version_id);
    }

    runtime.performance_access().reset_counters();
    for version_id in &versions[..versions.len() - 1] {
        let _ = runtime
            .read_truth()
            .read_version(execution, *version_id)
            .expect("read request remains live");
    }
    let stats = runtime.storage_access().storage_stats(execution).unwrap();

    assert_eq!(
        runtime
            .config()
            .visibility
            .cache_policy
            .recent_version_window,
        2
    );
    assert_eq!(stats.recent_visibility_cache_count, 2);
    assert!(stats.cached_visibility_version_count <= 3);
}

fn assert_recent_version_admission_candidate(
    runtime: &RelationalRuntime,
    version_id: crate::facade::identity::VersionId,
) {
    let read_path = runtime
        .read_truth()
        .inspect_version_read_path(version_id)
        .unwrap();
    assert!(read_path
        .entries
        .iter()
        .any(|entry| entry.code == DiagnosticCode::VisibilityCacheMissReconstructed));
    assert!(read_path.entries.iter().any(|entry| {
        entry.code == DiagnosticCode::VisibilityCacheRecentAdmissionCandidate
            && diagnostic_field(entry, "recent_admission_candidate")
                == &RelationalDiagnosticValue::Bool(true)
    }));
}

fn assert_version_read_path_has_cache_hit(
    runtime: &RelationalRuntime,
    version_id: crate::facade::identity::VersionId,
) {
    let read_path = runtime
        .read_truth()
        .inspect_version_read_path(version_id)
        .unwrap();
    assert!(read_path
        .entries
        .iter()
        .any(|entry| entry.code == DiagnosticCode::VisibilityCacheHit));
}

fn assert_evicted_version_is_not_cached(
    runtime: &RelationalRuntime,
    version_id: crate::facade::identity::VersionId,
) {
    let read_path = runtime
        .read_truth()
        .inspect_version_read_path(version_id)
        .unwrap();
    assert!(read_path.entries.iter().any(|entry| {
        entry.code == DiagnosticCode::SnapshotReadPathInspected
            && diagnostic_field(entry, "cached_visibility_state")
                == &RelationalDiagnosticValue::Bool(false)
            && diagnostic_field(entry, "recent_resident") == &RelationalDiagnosticValue::Bool(false)
    }));
}
