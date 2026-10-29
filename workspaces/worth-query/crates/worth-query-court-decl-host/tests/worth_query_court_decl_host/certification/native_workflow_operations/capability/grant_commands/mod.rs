pub(crate) use worth_query_testkit_certification_operation::facade::certification::native_workflow_operations::capability::grant_commands::*;

#[test]
fn product_stale_command_releases_each_managed_owner_before_native_progression() {
    use std::{num::NonZeroUsize, time::Duration};
    use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::request_scope;
    use WorthQueryCapabilityCommandAdvance as Advance;
    use WorthQueryCapabilityCommandDenial as Denial;

    let host = super::host::install();
    let runtime = host.application.runtime();
    let branch = runtime.current_world();
    let scope = request_scope();
    let issuer = super::host::authenticate(&host, 2, &scope);
    let request = runtime.request(&issuer, &scope);
    let record = runtime
        .with_host_advancement(|phase| {
            runtime
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
    let command = retain_revocation(&host, &issuer, &scope, 0x91765130);
    let mut command = {
        let _cleanup = runtime
            .schedule_next_managed_cleanup_on_worker_for_test()
            .unwrap();
        let pause =
            runtime.pause_after_application_candidate_preparation_for_test(NonZeroUsize::MIN);
        std::thread::scope(|threads| {
            let competitor = threads.spawn(|| {
                assert!(pause.wait_until_reached(Duration::from_secs(10)));
                runtime
                    .publish_native_field_write_for_test(
                        branch,
                        record,
                        DocumentRetentionField::reference(),
                        21_u64,
                        &scope,
                    )
                    .unwrap();
                pause.release();
            });
            let command = match command.advance(&request) {
                Advance::Progress(command) => command,
                _ => panic!("actual pre-World conflict retains the command and failed cleanup"),
            };
            competitor.join().unwrap();
            command
        })
    };
    let failure = command
        .managed_cleanup_failure()
        .expect("real failed managed run");
    let first_run = failure.run_identity().to_owned();
    let first_session = failure.session_identity();
    let kind = failure.kind();
    assert_eq!(
        kind,
        WorthQueryMutationCleanupFailureKind::Native(
            WorthQueryManagedRunCleanupFailureKind::BridgeFinalization(
                BridgeExecutionBasisFinalizationFailureKind::SignalRuntimeThreadAffinityViolation
            )
        )
    );
    assert_eq!(
        super::readback::grant_status(&host),
        1,
        "native ProductStale has not revoked the target"
    );
    assert_eq!(
        request
            .query(super::readback::ReadRetention {
                identity: DOCUMENT_IDENTITY.to_owned(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        21,
        "the independent native writer really moved the prepared product"
    );

    // A repeat would now revoke the grant. The same owner must refuse instead.
    command = match command.advance(&runtime.request(&issuer, &request_scope())) {
        Advance::Refused {
            command,
            denial: Denial::Cleanup(WorthQueryOperationCloseDenial::ManagedCleanup(actual)),
        } => {
            assert_eq!(actual, kind);
            command
        }
        _ => panic!("pending cleanup must precede any second native execution"),
    };
    let cancelled = WorthQueryCancellationSource::new();
    cancelled.cancel();
    let stopped = WorthQueryRequestScope::new(None, cancelled.token());
    assert!(
        matches!(
            command.release_cleanup(&runtime.request(&issuer, &stopped)),
            Err(Denial::Execution(_))
        ),
        "a stopped release preserves the original owner"
    );
    let retained = command.managed_cleanup_failure().unwrap();
    assert_eq!(retained.run_identity(), first_run);
    assert_eq!(retained.session_identity(), first_session);
    command = match command.advance(&request) {
        Advance::Refused {
            command,
            denial: Denial::Cleanup(WorthQueryOperationCloseDenial::ManagedCleanup(actual)),
        } => {
            assert_eq!(actual, kind);
            command
        }
        _ => panic!("failed release cannot enable execution or replace cleanup custody"),
    };
    assert_eq!(super::readback::grant_status(&host), 1);
    let released = command.release_cleanup(&request).unwrap();
    let native = released.managed_cleanup_receipt().unwrap().inspection();
    assert!(native.resources_released());
    assert_eq!(native.run_identity(), first_run);
    assert!(command.managed_cleanup_failure().is_none());

    // A fresh admitted retry may itself need managed cleanup. Its distinct
    // owner must be accessible, rather than stranded inside terminal evidence.
    let mut command = {
        let _cleanup = runtime
            .schedule_next_managed_cleanup_on_worker_for_test()
            .unwrap();
        match command.advance(&runtime.request(&issuer, &request_scope())) {
            Advance::Progress(command) => command,
            _ => panic!("fresh retry retains genuine performed-but-cleanup-uncertain custody"),
        }
    };
    assert_eq!(
        super::readback::grant_status(&host),
        0,
        "the freshly admitted retry actually revoked the original grant"
    );
    let second_run = command
        .managed_cleanup_failure()
        .unwrap()
        .run_identity()
        .to_owned();
    assert_ne!(second_run, first_run);
    let released = command.release_cleanup(&request).unwrap();
    let native = released.managed_cleanup_receipt().unwrap().inspection();
    assert!(native.resources_released());
    assert_eq!(native.run_identity(), second_run);
    assert!(command.managed_cleanup_failure().is_none());
    let completed = match command.advance(&runtime.request(&issuer, &request_scope())) {
        Advance::Completed(completed) => completed,
        _ => panic!("original keyed receipt settles without another revocation execution"),
    };
    assert!(matches!(
        completed.evidence(),
        WorthQueryCapabilityCommandCompletion::Recovered { .. }
    ));
    completed
        .close(&request)
        .unwrap_or_else(|_| panic!("both real managed owners were discharged"));
    assert_eq!(super::readback::grant_status(&host), 0);
}
