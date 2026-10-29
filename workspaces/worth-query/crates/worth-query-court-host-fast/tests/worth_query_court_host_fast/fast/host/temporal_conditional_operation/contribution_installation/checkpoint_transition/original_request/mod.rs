pub(crate) use worth_query_worlds::facade::host::temporal_conditional_operation::contribution_installation::checkpoint_transition::original_request::*;

#[test]
fn original_request_funded_native_transition_reads_and_reopens_target() {
    let (source, predecessor) = source();
    let retained = source.clone();
    let lease = primary_graph::query_test_execution_authority_for_test()
        .request_lease(request(CancellationToken::new()))
        .unwrap();
    let target = transition_original(
        ExecutionRequest::leased(&lease),
        source,
        predecessor,
        |writer, _| admission::author_record(writer, "original-funded-record", 137),
    )
    .unwrap();
    let checkpoint = target
        .with_funded_host_advancement(CapturePolicy::Execution(&lease), |phase| {
            target
                .capture_application_checkpoint(
                    &phase,
                    phase
                        .checked_execution_request_for(target.product_runtime())
                        .unwrap()
                        .allocation_policy(),
                )
                .unwrap()
        })
        .unwrap();
    assert!(checkpoint.charged_payload_bytes().is_some());
    assert_ne!(checkpoint.bytes(), retained.bytes());
    drop(target);
    admission::assert_target_record(checkpoint, 137);
}

#[test]
fn original_request_cancelled_author_domain_rejection_cannot_win_and_fresh_retry_works() {
    let (source, predecessor) = source();
    let retained = source.clone();
    let cancellation = CancellationSource::new();
    let lease = primary_graph::query_test_execution_authority_for_test()
        .request_lease(request(cancellation.token()))
        .unwrap();
    let denial = transition_original(ExecutionRequest::leased(&lease), source.clone(), predecessor, |writer, _| {
        admission::author_record(writer, "duplicate-original", 1)?;
        let domain = admission::author_record(writer, "duplicate-original", 2).expect_err("actual duplicate seed rejects");
        assert!(!matches!(domain.kind(), primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped { .. }));
        cancellation.cancel();
        Err(domain)
    }).err().unwrap();
    let InstallationDenial::Graph(cause) = denial else {
        panic!("no performed image after original stop: {denial:?}");
    };
    assert_cancelled(&cause);
    assert_eq!(source.bytes(), retained.bytes());
    let fresh = primary_graph::query_test_execution_authority_for_test()
        .request_lease(request(CancellationToken::new()))
        .unwrap();
    let repaired = transition_original(
        ExecutionRequest::leased(&fresh),
        source,
        source_predecessor(),
        |writer, _| admission::author_record(writer, "fresh-after-original-stop", 141),
    )
    .unwrap();
    let checkpoint = repaired
        .with_funded_host_advancement(CapturePolicy::Execution(&fresh), |phase| {
            repaired
                .capture_application_checkpoint(
                    &phase,
                    phase
                        .checked_execution_request_for(repaired.product_runtime())
                        .unwrap()
                        .allocation_policy(),
                )
                .unwrap()
        })
        .unwrap();
    admission::assert_target_record(checkpoint, 141);
}

#[test]
fn original_request_consumed_parent_work_refuses_actual_transition_then_fresh_retry() {
    let (source, predecessor) = source();
    let lease = primary_graph::query_test_execution_authority_for_test()
        .request_lease(request(CancellationToken::new()))
        .unwrap();
    let original = ExecutionRequest::leased(&lease);
    let mut authored = false;
    let (denial, _) = ExecutionWorkCeiling::new(Some(1024))
        .run_with_checkpoint(Some(&lease), None, |context| {
            let prefix = context
                .remaining_work()
                .expect("this native fixture installs a finite work ceiling")
                - 1;
            context.checkpoint_cost(prefix, prefix).unwrap();
            transition_original(original, source.clone(), predecessor, |_, _| {
                authored = true;
                Ok(())
            })
            .err()
            .expect("one remaining unit cannot decode a nonempty native checkpoint")
        })
        .unwrap();
    assert!(!authored);
    let InstallationDenial::Graph(cause) = denial else {
        panic!("pre-effect work refusal: {denial:?}");
    };
    assert_work(&cause);
    let fresh = primary_graph::query_test_execution_authority_for_test()
        .request_lease(request(CancellationToken::new()))
        .unwrap();
    let _target = transition_original(
        ExecutionRequest::leased(&fresh),
        source,
        source_predecessor(),
        |_, _| Ok(()),
    )
    .unwrap();
}

#[test]
fn original_request_serial_capture_and_nested_query_opening_refuse_before_authoring() {
    let (source, predecessor) = source();
    let serial = worth_query_worlds::facade::host::test_request_policy::serial_request();
    let mut authored = false;
    let denial = transition_original(
        ExecutionRequest::serial(&serial),
        source,
        predecessor,
        |_, _| {
            authored = true;
            Ok(())
        },
    )
    .err()
    .unwrap();
    assert!(!authored);
    let InstallationDenial::Graph(cause) = denial else {
        panic!("serial contract refusal: {denial:?}");
    };
    assert!(matches!(cause.kind(), primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
        kind: worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind::ResourceEnvelopeMismatch
    }));
    let lease = primary_graph::query_test_execution_authority_for_test()
        .request_lease(request(CancellationToken::new()))
        .unwrap();
    let host = application_installation::in_memory_program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        checkpoint::checkpoint_limits(),
        |_, graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "nested-bootstrap", 0, 1, true);
            Ok(())
        },
    )
    .unwrap();
    host.with_host_advancement(|_| {
        assert!(matches!(with_bootstrap_advancement_from_request(ExecutionRequest::leased(&lease), |_| ()),
            Err(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::NestedOpening)));
    }).unwrap();
}
