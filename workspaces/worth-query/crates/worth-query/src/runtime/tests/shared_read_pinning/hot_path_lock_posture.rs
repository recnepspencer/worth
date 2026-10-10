use super::*;

#[test]
fn shared_read_pin_hot_path_acquires_exact_zero_measured_locks() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, derived) =
        shared_read_pinning_workspace(request_execution, "shared-read.phase12.hot-path");

    for _ in 0..8 {
        let read = workspace
            .shared_read_context(request_execution)
            .expect("shared read context should mint without pin-registry locks");
        let artifact = read
            .published_derived_artifact(&derived)
            .expect("published artifact should mint");
        assert!(artifact.published_binding().is_some());
    }

    for step in 0..8 {
        insert_task(
            request_execution,
            &mut workspace,
            &format!("task-{}", step + 2),
            if step % 2 == 0 {
                "Task Two"
            } else {
                "Task One"
            },
        );
        let read = workspace
            .shared_read_context(request_execution)
            .expect("shared read context should mint after commit pressure");
        let artifact = read
            .published_derived_artifact(&derived)
            .expect("published artifact should mint after commit pressure");
        assert!(artifact.published_binding().is_some());
    }

    let counters = workspace.runtime.shared_read_counters();
    assert_eq!(counters.committed_read_hot_path_lock_count(), 0);
    assert_eq!(counters.orphaned_generation_count(), 0);
    assert_eq!(counters.unretired_pin_count(), 0);
}
