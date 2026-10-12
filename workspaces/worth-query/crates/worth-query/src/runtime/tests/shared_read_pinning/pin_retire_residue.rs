use super::*;

#[test]
fn shared_read_generation_pin_retire_schedule_returns_exact_zero_residue() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, derived) =
        shared_read_pinning_workspace(request_execution, "shared-read.phase12.residue");

    {
        let old_read = workspace
            .shared_read_context(request_execution)
            .expect("old read context should mint");
        let old_artifact_before_commit = old_read
            .published_derived_artifact(&derived)
            .expect("old artifact should mint");
        insert_task(request_execution, &mut workspace, "task-2", "Task Two");
        let new_read = workspace
            .shared_read_context(request_execution)
            .expect("new read context should mint");
        let old_artifact_after_commit = old_read
            .published_derived_artifact(&derived)
            .expect("old pinned context should still resolve its generation after commit pressure");
        let new_artifact = new_read
            .published_derived_artifact(&derived)
            .expect("new artifact should mint");
        let pinning = workspace.runtime.shared_read_pinning_diagnostics();

        assert_ne!(old_read.snapshot_identity(), new_read.snapshot_identity());
        assert_eq!(
            old_artifact_before_commit
                .published_binding()
                .expect("old artifact before commit should be published")
                .binding_for_reporting(),
            old_artifact_after_commit
                .published_binding()
                .expect("old artifact after commit should be published")
                .binding_for_reporting()
        );
        assert_ne!(
            old_artifact_after_commit
                .published_binding()
                .expect("old artifact after commit should be published")
                .binding_for_reporting(),
            new_artifact
                .published_binding()
                .expect("new artifact should be published")
                .binding_for_reporting()
        );
        assert!(
            pinning
                .generations()
                .iter()
                .any(
                    |generation| generation.snapshot_identity() == old_read.snapshot_identity()
                        && generation.is_retired()
                        && !generation.is_invalidated()
                        && generation.pin_count() > 0
                ),
            "old generation must be explicitly retired but retained while old leases exist"
        );
        assert!(
            pinning
                .generations()
                .iter()
                .any(
                    |generation| generation.snapshot_identity() == new_read.snapshot_identity()
                        && generation.is_current()
                        && !generation.is_invalidated()
                        && !generation.is_retired()
                ),
            "new generation must become the current committed read generation"
        );
    }

    insert_task(request_execution, &mut workspace, "task-3", "Task Three");
    assert!(pinning_phase_twelve_counters_are_closed(&workspace.runtime));
}
