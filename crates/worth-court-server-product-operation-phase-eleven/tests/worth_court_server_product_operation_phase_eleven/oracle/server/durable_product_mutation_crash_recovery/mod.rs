#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint, worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor, worth_server::WorthServerProductOperationOutcome, worth_server::WorthServerProductOperationSurfaceDenialCode, crate::durable_product_mutation_crash_recovery::ordinary_context::assert_failed_at, worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server, crate::durable_product_mutation_crash_recovery::ordinary_context::crash_payload, worth_testkit_server::shared_support_durable_product_mutation::driver::execute, worth_testkit_server::shared_support_durable_product_mutation::runtime::session};

#[test]
fn crash_boundary_matrix_resolves_without_duplicate_product_effects() {
    for crash in [
        worth_testkit_server::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::BeforeIntent,
        worth_testkit_server::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::AfterIntent,
        worth_testkit_server::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::AfterMutationBeforeCommit,
    ] {
        let executor = worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::default();
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
            worth_server::WorthServerProductOperationOutcome::Success(_)
        ));
        assert_eq!(executor.commit_count(), 1);
    }

    let executor = worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::default();
    let first_server = build_server(&executor);
    let payload = crash_payload(worth_testkit_server::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::AfterCommitBeforeAcknowledgment);
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
        worth_server::WorthServerProductOperationSurfaceDenialCode::Indeterminate
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
