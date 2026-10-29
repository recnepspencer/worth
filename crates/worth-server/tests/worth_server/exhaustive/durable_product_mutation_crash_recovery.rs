#[allow(unused_imports)]
pub(crate) use crate::fast::durable_product_mutation_crash_recovery::*;

#[test]
fn crash_boundary_matrix_resolves_without_duplicate_product_effects() {
    for crash in [
        DurableMutationCrashPoint::BeforeIntent,
        DurableMutationCrashPoint::AfterIntent,
        DurableMutationCrashPoint::AfterMutationBeforeCommit,
    ] {
        let executor = TestDurableProductExecutor::default();
        let first_server = build_server(&executor);
        let payload = crash_payload(crash);
        let interrupted = execute(
            &session(&first_server, "tenant-a", "workspace-42"),
            "product.host_connection.upsert",
            "product.host-connection.upsert.v1",
            payload.clone(),
            "basis:0",
            "crash-matrix-key",
        )
        .expect("pre-commit crash should return a typed failed operation");
        assert_failed_at(&interrupted, crash);
        assert_eq!(executor.commit_count(), 0);
        drop(first_server);

        let rebuilt_server = build_server(&executor);
        let retried = execute(
            &session(&rebuilt_server, "tenant-a", "workspace-42"),
            "product.host_connection.upsert",
            "product.host-connection.upsert.v1",
            payload,
            "basis:0",
            "crash-matrix-key",
        )
        .expect("rolled-back crash point should permit one fresh commit");
        assert!(matches!(
            retried.outcome(),
            WorthServerProductOperationOutcome::Success(_)
        ));
        assert_eq!(executor.commit_count(), 1);
    }

    let executor = TestDurableProductExecutor::default();
    let first_server = build_server(&executor);
    let payload = crash_payload(DurableMutationCrashPoint::AfterCommitBeforeAcknowledgment);
    let indeterminate = execute(
        &session(&first_server, "tenant-a", "workspace-42"),
        "product.host_connection.upsert",
        "product.host-connection.upsert.v1",
        payload.clone(),
        "basis:0",
        "post-commit-crash-key",
    )
    .expect_err("post-commit acknowledgment loss must remain indeterminate");
    assert_eq!(
        indeterminate.code(),
        WorthServerProductOperationSurfaceDenialCode::Indeterminate
    );
    assert!(indeterminate
        .facts()
        .and_then(|facts| facts.recovery_handle())
        .is_some());
    assert_eq!(executor.commit_count(), 1);
    drop(first_server);

    let rebuilt_server = build_server(&executor);
    let retried = execute(
        &session(&rebuilt_server, "tenant-a", "workspace-42"),
        "product.host_connection.upsert",
        "product.host-connection.upsert.v1",
        payload,
        "basis:0",
        "post-commit-crash-key",
    )
    .expect("post-commit retry should resolve the persisted completion");
    assert!(retried.retry_diagnostics().is_previously_committed());
    assert_eq!(executor.commit_count(), 1);
}
