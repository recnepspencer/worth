use super::*;

#[test]
fn shared_read_context_is_send_sync_and_resolves_in_scoped_thread() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<crate::runtime::WorthQuerySharedReadContext>();
    assert_send_sync::<crate::runtime::WorthQueryPublishedDerivedArtifactHandle>();

    let (workspace, derived) =
        shared_read_pinning_workspace(request_execution, "shared-read.phase13.send-sync");
    let read = workspace
        .shared_read_context(request_execution)
        .expect("shared read context should mint");
    let serial_binding = read
        .published_derived_artifact(&derived)
        .expect("serial artifact should resolve")
        .published_binding()
        .expect("serial artifact should be published")
        .binding_for_reporting()
        .to_string();
    let sibling_read = read.clone();

    let (first_thread_binding, second_thread_binding) = std::thread::scope(|scope| {
        let first_handle = scope.spawn(|| {
            read.published_derived_artifact(&derived)
                .expect("scoped thread should resolve through the real shared-read context")
                .published_binding()
                .expect("thread artifact should be published")
                .binding_for_reporting()
                .to_string()
        });
        let second_handle = scope.spawn(|| {
            sibling_read
                .published_derived_artifact(&derived)
                .expect("second scoped thread should resolve through a cloned real context")
                .published_binding()
                .expect("second thread artifact should be published")
                .binding_for_reporting()
                .to_string()
        });
        let first_thread_binding = first_handle
            .join()
            .expect("first scoped thread should finish");
        assert_eq!(first_thread_binding, serial_binding);
        let second_thread_binding = second_handle
            .join()
            .expect("second scoped thread should finish");
        assert_eq!(second_thread_binding, serial_binding);
        (first_thread_binding, second_thread_binding)
    });
    let portability = WorthQuerySharedReadPortabilityEvidence::proven(evidence_digest(
        "shared-read-send-sync-scoped-thread",
        [
            ("serial_binding", serial_binding),
            ("first_thread_binding", first_thread_binding),
            ("second_thread_binding", second_thread_binding),
        ],
    ));
    assert!(portability.proven_by_scoped_thread());
}
