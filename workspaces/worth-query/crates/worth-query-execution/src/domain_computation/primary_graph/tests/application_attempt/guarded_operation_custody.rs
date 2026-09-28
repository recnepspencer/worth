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
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let program = admitted_program(&world, &principal, &account, &request, "committed");
    let transition = [73; 32];
    let binding = idempotency(74, 75).bind_guarded_workflow_effect(&transition);

    assert!(matches!(
        world
            .application
            .resolve_admitted_guarded_workflow_operation_custody(&admission, binding, &transition)
            .unwrap(),
        WorthQueryGuardedWorkflowOperationCustody::Unseen
    ));
    let WorthQueryApplicationCommitOutcome::Committed(committed) = world
        .application
        .compare_and_commit_application(program, binding)
    else {
        panic!("the guarded mutation must commit");
    };
    let WorthQueryGuardedWorkflowOperationCustody::Committed(recovered) = world
        .application
        .resolve_admitted_guarded_workflow_operation_custody(&admission, binding, &transition)
        .unwrap()
    else {
        panic!("exact owner custody must recover the guarded commit");
    };
    assert!(recovered.is_same_authoritative_commit(&committed));

    let drift = idempotency(76, 77).bind_guarded_workflow_effect(&transition);
    assert!(matches!(
        world
            .application
            .resolve_admitted_guarded_workflow_operation_custody(&admission, drift, &transition)
            .unwrap(),
        WorthQueryGuardedWorkflowOperationCustody::IntentDrift
    ));
    let denial = world
        .application
        .resolve_admitted_guarded_workflow_operation_custody(&admission, binding, &[78; 32])
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission
    );
}

#[test]
fn guarded_operation_custody_retains_world_issued_unpublished_recovery() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let program = admitted_program(&world, &principal, &account, &request, "unpublished");
    let transition = [79; 32];
    let binding = idempotency(80, 81).bind_guarded_workflow_effect(&transition);

    world.application.fail_next_durable_append_for_test();
    let WorthQueryApplicationCommitOutcome::ProductUnpublished(partial) = world
        .application
        .compare_and_commit_application(program, binding)
    else {
        panic!("the World must issue unpublished recovery material");
    };
    let expected = partial.into_recovery().record_handle().clone();
    let WorthQueryGuardedWorkflowOperationCustody::ProductUnpublished(actual) = world
        .application
        .resolve_admitted_guarded_workflow_operation_custody(&admission, binding, &transition)
        .unwrap()
    else {
        panic!("the exact owner-unpublished outcome must retain World recovery");
    };
    assert_eq!(actual, expected);

    let drift = idempotency(82, 83).bind_guarded_workflow_effect(&transition);
    assert!(matches!(
        world
            .application
            .resolve_admitted_guarded_workflow_operation_custody(&admission, drift, &transition)
            .unwrap(),
        WorthQueryGuardedWorkflowOperationCustody::IntentDrift
    ));
}

#[test]
fn reserved_publication_custody_is_indeterminate_until_its_owner_releases_it() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let transition = [84; 32];
    let binding = idempotency(85, 86).bind_guarded_workflow_effect(&transition);
    let reservation = world
        .application
        .primary_provider
        .reserve_application_publication_recovery(world.selected_product().product().observation())
        .expect("the owner reserves the exact product incarnation");
    let WorthQueryGuardedWorkflowOperationCustody::Indeterminate(denial) = world
        .application
        .resolve_admitted_guarded_workflow_operation_custody(&admission, binding, &transition)
        .unwrap()
    else {
        panic!("a reserved but not yet installed publication cannot be called unseen");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable
    );
    drop(reservation);
    assert!(matches!(
        world
            .application
            .resolve_admitted_guarded_workflow_operation_custody(&admission, binding, &transition)
            .unwrap(),
        WorthQueryGuardedWorkflowOperationCustody::Unseen
    ));
}
