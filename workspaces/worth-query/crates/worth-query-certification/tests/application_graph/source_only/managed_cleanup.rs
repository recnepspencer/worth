//! Actual pre-World native candidate races preserve terminal source custody.
use super::{fixture::*, readback::*, schema::*};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::primary_graph::BridgeExecutionBasisFinalizationFailureKind;
use worth_query_host::facade::{
    application_entry::*, declaration::application_schema::*, primary_graph::*,
};

#[test]
fn managed_product_stale_retains_native_failure_until_owner_thread_close() {
    let host = install();
    let branch = host.current_world();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let (original_head, record, original) = {
        let request = host.runtime().request(&principal, &scope);
        let original_head = request.retain_current_read().unwrap();
        let record = host
            .runtime()
            .with_host_advancement(|phase| {
                host.runtime()
                    .on_branch(branch)
                    .select(&phase)
                    .unwrap()
                    .resolve_entity(
                        &phase,
                        DocumentIdentityField::reference(),
                        DOCUMENT_IDENTITY.to_owned(),
                        &scope,
                        WorthQueryPrincipalResolutionMode::Certification,
                    )
                    .unwrap()
                    .relational_record_identity_parts()
            })
            .unwrap();
        let original = request
            .mutate(OriginalRetention {
                input: SetRetentionInput {
                    identity: DOCUMENT_IDENTITY.to_owned(),
                    retention_days: 19,
                },
            })
            .without_source()
            .preconditions(
                TypedMutationPreconditions::new()
                    .expect_fact(
                        DocumentIdentityField::reference(),
                        ApplicationEncodedScalarValue::try_new(DOCUMENT_IDENTITY.to_owned())
                            .unwrap(),
                    )
                    .expect_fact(
                        DocumentRetentionField::reference(),
                        ApplicationEncodedScalarValue::try_new(7_u64).unwrap(),
                    ),
            )
            .idempotency_owned(0x9176_6019)
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
            .retain_operation::<SourceOnlyRetentionProgram, RetentionAction>(&host)
            .unwrap_or_else(|(_, cause)| panic!("original descriptor: {cause:?}"));
        (original_head, record, original)
    };
    let _cleanup = host
        .runtime()
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let pause = host
        .runtime()
        .pause_after_application_candidate_preparation_for_test(NonZeroUsize::MIN);
    let terminal = std::thread::scope(|threads| {
        let competitor = threads.spawn(|| {
            assert!(pause.wait_until_reached(Duration::from_secs(10)));
            assert_eq!(host.contacts.counts(), (1, 1));
            host.runtime()
                .publish_native_field_write_for_test(
                    branch,
                    record,
                    DocumentRetentionField::reference(),
                    21_u64,
                    &scope,
                )
                .unwrap();
            let winner = observe_head(host.runtime(), branch);
            pause.release();
            winner
        });
        let terminal = match original.advance(&host.runtime().request(&principal, &scope)) {
            WorthQuerySourceOnlyAdvance::Terminal(owner) => owner,
            _ => panic!("real stale business outcome retains cleanup custody"),
        };
        let winner = competitor.join().unwrap();
        assert_native_failure(terminal.managed_cleanup_failure().unwrap());
        assert!(
            matches!(terminal.native_outcome(), Some(WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::ProductStale(stale)))
            if stale.expected_product().selected_commit() == original_head.selected_commit()
                && stale.observed_product().is_none())
        );
        assert_eq!(
            observe_head(host.runtime(), branch).selected_commit(),
            winner.selected_commit()
        );
        assert_eq!(read_retention(host.runtime(), branch), 21);
        terminal
    });
    assert_eq!(host.contacts.counts(), (1, 1));
    assert_eq!(host.runtime().active_application_attempts_for_test(), 0);
    assert_eq!(
        host.runtime().world_active_publication_attempts_for_test(),
        0
    );
    assert!(host
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    let original_run = terminal
        .managed_cleanup_failure()
        .unwrap()
        .run_identity()
        .to_owned();
    let session = terminal
        .managed_cleanup_failure()
        .unwrap()
        .session_identity();
    let (terminal, cause) = std::thread::scope(|threads| {
        threads
            .spawn(|| {
                let scope = request_scope();
                let principal = authenticate_operator(host.installed_schema(), &scope);
                terminal.close(&host.runtime().request(&principal, &scope))
            })
            .join()
            .unwrap()
    })
    .err()
    .expect("actual Signal owner refuses foreign-thread cleanup");
    assert!(matches!(
        cause,
        WorthQueryOperationCloseDenial::ManagedCleanup(_)
    ));
    assert_eq!(
        terminal
            .managed_cleanup_failure()
            .unwrap()
            .session_identity(),
        session
    );
    let cancellation = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    cancellation.cancel();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancellation.token(),
        );
    let terminal = {
        let request = host.runtime().request(&principal, &stopped);
        let (terminal, cause) = terminal
            .close(&request)
            .err()
            .expect("close cannot bypass a stopped scope");
        assert!(matches!(
            cause,
            WorthQueryOperationCloseDenial::Execution(_)
        ));
        assert!(matches!(
            terminal.native_outcome(),
            Some(WorthQueryApplicationMutationOutcome::Commit(
                WorthQueryApplicationUncommitted::ProductStale(_)
            ))
        ));
        terminal
    };
    drop(principal);
    drop(scope);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    let closed = terminal
        .close(&host.runtime().request(&fresh_principal, &fresh_scope))
        .unwrap_or_else(|(_, cause)| panic!("fresh terminal close: {cause:?}"));
    let native = closed.managed_cleanup_receipt().unwrap().inspection();
    assert!(native.resources_released());
    assert_eq!(native.run_identity(), original_run);

    assert_eq!(host.contacts.counts(), (1, 1));
    assert_eq!(read_retention(host.runtime(), branch), 21);
}

fn assert_native_failure(failure: &WorthQueryMutationCleanupFailure) {
    assert_eq!(
        failure.kind(),
        WorthQueryMutationCleanupFailureKind::Native(
            WorthQueryManagedRunCleanupFailureKind::BridgeFinalization(
                BridgeExecutionBasisFinalizationFailureKind::SignalRuntimeThreadAffinityViolation
            )
        )
    );
}
#[path = "managed_cleanup/retained_capacity.rs"]
mod retained_capacity;

#[test]
fn managed_cleanup_discharge_does_not_settle_uncertain_business_outcome() {
    let host = install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let _guard = host
        .runtime()
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let mut operation = match operation(&host, &request, 19, 0x9176_6719).advance(&request) {
        WorthQuerySourceOnlyAdvance::Progress(operation) => operation,
        _ => panic!("performed source with failed cleanup retains genuine uncertainty"),
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::Indeterminate
    );
    assert_native_failure(operation.managed_cleanup_failure().unwrap());
    assert_eq!(read_retention(host.runtime(), host.current_world()), 19);
    let contacts = host.contacts.counts();
    assert_eq!(contacts, (1, 1));
    let stop = retained_capacity::authentication_stop();
    assert!(matches!(
        operation.release_cleanup(&host.runtime().request(&principal, &stop)),
        Err(WorthQueryOperationCloseDenial::Execution(_))
    ));
    assert!(operation.managed_cleanup_failure().is_some());
    let released = operation.release_cleanup(&request).unwrap();
    assert!(released
        .managed_cleanup_receipt()
        .unwrap()
        .inspection()
        .resources_released());
    assert!(operation.managed_cleanup_failure().is_none());
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::Indeterminate
    );
    assert_eq!(host.contacts.counts(), contacts);
    let completed = match operation.advance(&request) {
        WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        _ => panic!("only the real keyed recovery resolves the committed source"),
    };
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Recovered {
            initial_posture: Some(WorthQueryApplicationUncommitted::Indeterminate(_)),
            ..
        }
    ));
    completed
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("actual cleanup discharged: {denial:?}"));
    assert_eq!(host.contacts.counts(), contacts);
    assert_eq!(read_retention(host.runtime(), host.current_world()), 19);
}

#[path = "managed_cleanup/unpublished.rs"]
mod unpublished;

#[test]
fn managed_failure_abandonment_transfers_actual_collection_and_installed_keepalive() {
    let host = install();
    let contacts = host.contacts.clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let (mut runtime, mut cleanup, _guard) = {
        let request = host.runtime().request(&principal, &scope);
        let _guard = host
            .runtime()
            .schedule_next_managed_cleanup_on_worker_for_test()
            .unwrap();
        let operation = match operation(&host, &request, 19, 0x9176_6819).advance(&request) {
            WorthQuerySourceOnlyAdvance::Progress(operation) => operation,
            _ => panic!("real cleanup refusal preserves the source owner"),
        };
        let (runtime, cleanup) = operation.into_runtime_for_cleanup();
        assert_native_failure(cleanup.managed_failure().unwrap());
        (runtime, cleanup, _guard)
    };
    drop(principal);
    drop(scope);
    drop(host);
    assert_eq!(
        runtime.close_conditional_runtime().unwrap_err().kind(),
        WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained
    );
    let fresh_scope = request_scope();
    runtime
        .with_application_advancement(&fresh_scope, |phase| {
            cleanup.close_in_advancement(&phase, runtime.runtime())
        })
        .unwrap()
        .unwrap();
    assert!(cleanup.managed_failure().is_none());
    assert_eq!(
        read_retention(runtime.runtime(), runtime.current_world()),
        19
    );
    assert_eq!(contacts.counts(), (1, 1));
    // Explicit runtime shutdown is separate from settling the abandoned business meaning.
    runtime.close_conditional_runtime().unwrap();
}
