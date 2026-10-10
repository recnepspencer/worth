use super::*;

#[test]
fn published_artifact_generations_retire_with_pinned_read_generations() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, derived) =
        shared_read_pinning_workspace(request_execution, "shared-read.phase12.artifacts");

    let old_read = workspace
        .shared_read_context(request_execution)
        .expect("old read context should mint");
    let old_artifact = old_read
        .published_derived_artifact(&derived)
        .expect("old artifact should mint");
    let old_ordinal = generation_ordinal(&workspace.runtime, old_read.snapshot_identity());

    insert_task(request_execution, &mut workspace, "task-2", "Task Two");
    let new_read = workspace
        .shared_read_context(request_execution)
        .expect("new read context should mint");
    let new_ordinal = generation_ordinal(&workspace.runtime, new_read.snapshot_identity());
    assert_ne!(old_ordinal, new_ordinal);

    let retained = workspace.runtime.published_artifact_diagnostics();
    assert!(retained.contains_generation(old_ordinal));
    assert!(retained.contains_generation(new_ordinal));
    assert_eq!(retained.retained_generation_count(), 2);
    assert!(retained.generations().iter().all(|generation| {
        generation.artifact_count() > 0
            && !generation
                .snapshot_identity()
                .evidence_identity()
                .as_str()
                .is_empty()
    }));

    drop(old_artifact);
    drop(old_read);
    insert_task(request_execution, &mut workspace, "task-3", "Task Three");

    let drained = workspace.runtime.published_artifact_diagnostics();
    assert!(!drained.contains_generation(old_ordinal));
    assert!(drained.contains_generation(new_ordinal));
    assert!(
        drained.counters().dropped_generation_count() > 0,
        "published artifact registry must account for dropped generations"
    );
    assert!(drained
        .generations()
        .iter()
        .any(|generation| generation.ordinal() == new_ordinal && generation.artifact_count() > 0));
    drop(new_read);
    insert_task(request_execution, &mut workspace, "task-4", "Task Four");
    assert!(pinning_phase_twelve_counters_are_closed(&workspace.runtime));
}
