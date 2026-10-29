pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::publication_limit::*;

#[test]
fn explicit_publication_limit_survives_checkpoint_installation_and_enforces_commit_boundary() {
    let host_request =
        worth_query_testkit_host::facade::host::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = application_installation::in_memory_program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        (configuration(),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        limits(),
        |_phase, graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "original", 0, 1, true);
            Ok(())
        },
    )
    .expect("ordinary source installation");
    let checkpoint = source
        .with_host_advancement(|host_phase| {
            source
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .expect("installed host call opens");
    let original = read_intent(&source);
    drop(source);
    for maximum in [1, 65_536] {
        let application = application_installation::in_memory_program_from_checkpoint(
            validated_program(),
            TemporalHostSchema::declaration().unwrap(),
            (configuration(),),
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            limits()
                .with_maximum_publication_records(std::num::NonZeroUsize::new(maximum).unwrap()),
            checkpoint.clone(),
        )
        .expect("a record budget governs new commits, not valid checkpoint recovery");
        assert_eq!(read_intent(&application), original);
        let installed = application.conditional::<TemporalConditional>().unwrap();
        let installed = installed.as_ref().as_ref().unwrap();
        let outcome = change_input(
            request_execution,
            &application,
            &installed.invariant,
            application.current_world(),
            "changed",
        );
        if maximum == 1 {
            assert_patch_budget_denial(outcome);
            assert_eq!(
                read_intent(&application),
                original,
                "rejected publication leaves authoritative fields unchanged"
            );
            assert_source_change_key_absent(&application);
            assert_patch_budget_denial(change_input(
                request_execution,
                &application,
                &installed.invariant,
                application.current_world(),
                "changed",
            ));
            assert_source_change_key_absent(&application);
            assert_eq!(read_intent(&application), original);
        } else {
            outcome
                .require_committed()
                .expect("same real command commits within its explicit allowance");
            let changed = read_intent(&application);
            assert_eq!(changed.input, "changed");
            assert_eq!(changed.revision, 2);
        }
    }
}
