use super::*;

#[test]
fn invalidated_shared_read_context_fails_typed_without_rebinding() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, derived) =
        shared_read_pinning_workspace(request_execution, "shared-read.phase13.stale");
    let read = workspace
        .shared_read_context(request_execution)
        .expect("shared read context should mint");
    let stale_basis = read.inspect_basis();
    let old_binding = read
        .published_derived_artifact(&derived)
        .expect("old artifact should resolve before invalidation")
        .published_binding()
        .expect("old artifact should be published")
        .binding_for_reporting()
        .to_string();

    insert_task(request_execution, &mut workspace, "task-2", "Task Two");
    workspace
        .runtime
        .invalidate_shared_read_snapshot_for_certification(stale_basis.snapshot_identity());
    let newer = workspace
        .shared_read_context(request_execution)
        .expect("new context should remint explicitly");
    let newer_binding = newer
        .published_derived_artifact(&derived)
        .expect("new artifact should resolve")
        .published_binding()
        .expect("new artifact should be published")
        .binding_for_reporting()
        .to_string();

    let error = read
        .published_derived_artifact(&derived)
        .expect_err("invalidated context must fail typed");
    match error {
        crate::runtime::WorthQueryRuntimeError::SharedReadStaleBasis { snapshot_identity } => {
            assert_eq!(&snapshot_identity, stale_basis.snapshot_identity());
            let denial_evidence =
                WorthQuerySharedReadStaleBasisDenialEvidence::proven(evidence_digest(
                    "shared-read-stale-basis-denial",
                    [
                        (
                            "snapshot_identity",
                            snapshot_identity.evidence_identity().as_str().to_string(),
                        ),
                        ("old_binding", old_binding.clone()),
                        ("newer_binding", newer_binding.clone()),
                    ],
                ));
            assert!(denial_evidence.proven_by_typed_denial());
        }
        other => panic!("expected shared-read stale basis error, got {other:?}"),
    }
    assert_ne!(old_binding, newer_binding);
}
