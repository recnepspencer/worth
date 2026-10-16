use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryGuardedWorkflowOperationCustody;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyResolutionDenialKind,
};

#[test]
fn guarded_operation_custody_recovers_only_the_exact_committed_mutation() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
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
                .selected_product(&(phase).test_host_request())
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
            let host_custody = phase.test_host_request();
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
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the guarded mutation must commit");
    };
    for completed in [None, Some(&committed)] {
        let WorthQueryGuardedWorkflowOperationCustody::Committed(recovered) = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
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
            .expect("the fixture read owns its installed host call")
        else {
            panic!("exact owner custody must recover the guarded commit");
        };
        assert!(recovered.is_same_authoritative_commit(&committed));
        assert_eq!(recovered.principal_scope(), committed.principal_scope());
    }

    let drift = idempotency(76, 77).bind_guarded_workflow_effect(&transition);
    for completed in [None, Some(&committed)] {
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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
fn guarded_operation_custody_retains_world_issued_unpublished_recovery() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
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
                .selected_product(&(phase).test_host_request())
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
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "unpublished",
            )
        })
        .expect("the fixture read owns its installed host call");
    let transition = [79; 32];
    let binding = idempotency(80, 81).bind_guarded_workflow_effect(&transition);

    world
        .application
        .fail_next_durable_append_for_test(request_execution);
    let WorthQueryApplicationCommitOutcome::ProductUnpublished(partial) =
        world.application.compare_and_commit_application(
            program,
            binding,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the World must issue unpublished recovery material");
    };
    let expected = partial.into_recovery().record_handle().clone();
    let WorthQueryGuardedWorkflowOperationCustody::ProductUnpublished(actual) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
        panic!("the exact owner-unpublished outcome must retain World recovery");
    };
    assert_eq!(actual, expected);

    let drift = idempotency(82, 83).bind_guarded_workflow_effect(&transition);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(matches!(
                world
                    .application
                    .resolve_admitted_guarded_workflow_operation_custody(
                        &host_custody.borrowed_request_for_test(),
                        &admission,
                        drift,
                        &transition,
                        None,
                    )
                    .unwrap(),
                WorthQueryGuardedWorkflowOperationCustody::IntentDrift
            ))
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn reserved_publication_custody_is_indeterminate_until_its_owner_releases_it() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
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
                .selected_product(&(phase).test_host_request())
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
            world
                .application
                .primary_provider
                .reserve_application_publication_recovery(
                    world
                        .selected_product(&(phase).test_host_request())
                        .product()
                        .observation(),
                )
                .expect("the owner reserves the exact product incarnation")
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryGuardedWorkflowOperationCustody::Indeterminate(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
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
