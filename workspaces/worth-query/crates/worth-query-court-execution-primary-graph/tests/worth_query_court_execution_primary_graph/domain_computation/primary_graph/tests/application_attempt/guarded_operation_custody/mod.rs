use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::TouchAccountOperation;
use worth_query_execution::publication_boundary::workflow_advance::WorthQueryGuardedWorkflowOperationCustody;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyResolutionDenialKind;

#[test]
fn guarded_operation_custody_recovers_only_the_exact_committed_mutation() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "committed")
        })
        .expect("the fixture read owns its installed host call");
    let transition = [73; 32];
    let binding = idempotency(74, 75).bind_guarded_workflow_effect(&transition);

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(matches!(
                world
                    .application
                    .resolve_admitted_guarded_workflow_operation_custody(
                        &host_custody.borrowed_request_for_test(),
                        &admission,
                        binding,
                        &transition,
                        None,
                    )
                    .unwrap(),
                WorthQueryGuardedWorkflowOperationCustody::Unseen
            ))
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Committed(committed) =
        world.application.compare_and_commit_application(
            program,
            binding,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the guarded mutation must commit");
    };
    for completed in [None, Some(&committed)] {
        let recovered = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                world
                    .application
                    .resolve_admitted_guarded_workflow_operation_custody(
                        &host_custody.borrowed_request_for_test(),
                        &admission,
                        binding,
                        &transition,
                        completed,
                    )
                    .unwrap()
            })
            .expect("the fixture read owns its installed host call");
        match (completed, recovered) {
            (None, WorthQueryGuardedWorkflowOperationCustody::RecoveredCommitted(original)) => {
                crate::domain_computation::primary_graph::tests::fixture::recorded_receipt::assert_original(&original, &committed);
            }
            (Some(_), WorthQueryGuardedWorkflowOperationCustody::Committed(owned)) => {
                assert!(owned.is_same_authoritative_commit(&committed));
                assert_eq!(owned.principal_scope(), committed.principal_scope());
            }
            (_, other) => {
                panic!("existing live custody stays distinct from reconstructed data: {other:?}")
            }
        }
    }

    let drift = idempotency(76, 77).bind_guarded_workflow_effect(&transition);
    for completed in [None, Some(&committed)] {
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                assert!(matches!(
                    world
                        .application
                        .resolve_admitted_guarded_workflow_operation_custody(
                            &host_custody.borrowed_request_for_test(),
                            &admission,
                            drift,
                            &transition,
                            completed,
                        )
                        .unwrap(),
                    WorthQueryGuardedWorkflowOperationCustody::IntentDrift
                ))
            })
            .expect("the fixture read owns its installed host call");
    }
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .resolve_admitted_guarded_workflow_operation_custody(
                    &host_custody.borrowed_request_for_test(),
                    &admission,
                    binding,
                    &[78; 32],
                    None,
                )
                .unwrap_err()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission
    );
}

#[test]
fn reserved_publication_custody_is_indeterminate_until_its_owner_releases_it() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let transition = [84; 32];
    let binding = idempotency(85, 86).bind_guarded_workflow_effect(&transition);
    let reservation = world
        .application
        .with_host_advancement(|phase| {
            (*(*world.application.runtime_for_test()).primary_provider_for_test())
                .reserve_application_publication_recovery_for_test(
                    world
                        .selected_product(&(phase).test_host_request_for_test())
                        .product()
                        .observation_for_test(),
                    None,
                )
                .expect("the owner reserves the exact product incarnation")
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryGuardedWorkflowOperationCustody::Indeterminate(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .resolve_admitted_guarded_workflow_operation_custody(
                    &host_custody.borrowed_request_for_test(),
                    &admission,
                    binding,
                    &transition,
                    None,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a reserved but not yet installed publication cannot be called unseen");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable
    );
    drop(reservation);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(matches!(
                world
                    .application
                    .resolve_admitted_guarded_workflow_operation_custody(
                        &host_custody.borrowed_request_for_test(),
                        &admission,
                        binding,
                        &transition,
                        None,
                    )
                    .unwrap(),
                WorthQueryGuardedWorkflowOperationCustody::Unseen
            ))
        })
        .expect("the fixture read owns its installed host call");
}
