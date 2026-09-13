use super::program_fixture::admitted_program;
use super::{
    authenticated_principal, idempotency, installed_authorization_world, live_scope,
    resolved_account,
};
use crate::domain_computation::primary_graph::tests::fixture::MultiTouchOperation;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitTerminalKind,
    WorthQueryApplicationIdempotencyResolution,
};

#[test]
fn equivalent_retry_recovers_original_receipt_while_intent_drift_is_denied() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let first = admitted_program(&world, &principal, &account, &request, "committed");

    let WorthQueryApplicationCommitOutcome::Committed(mut original) = world
        .application
        .compare_and_commit_application(first, idempotency(9, 7))
    else {
        panic!("the first idempotent application attempt must commit");
    };
    let mut descriptive_copy = original.clone();
    assert!(descriptive_copy
        .take_performed_relational_product_change()
        .is_none());
    let performed = original
        .take_performed_relational_product_change()
        .expect("the fresh performed publication carries one delivery witness");
    assert!(original
        .take_performed_relational_product_change()
        .is_none());

    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "committed", &request);
    let retry = admitted_program(&world, &principal, &account, &request, "committed");
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(mut recovered) = world
        .application
        .compare_and_commit_application(retry, idempotency(9, 7))
    else {
        panic!("an equivalent retry must recover the original commit");
    };
    assert!(recovered.is_same_authoritative_commit(&original));
    assert_eq!(
        original.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Executed
    );
    assert_eq!(
        recovered.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
    assert_eq!(recovered.terminal().attempt_resources_released(), None);
    assert!(recovered
        .take_performed_relational_product_change()
        .is_none());
    let first_root = world
        .application
        .granular_invalidation_installation()
        .retain_product_shared_root();
    let second_root = world
        .application
        .granular_invalidation_installation()
        .retain_product_shared_root();
    assert!(first_root.is_same_root_as(&second_root));
    assert!(first_root.accepts_performed_change(&performed));

    let commit = performed.product_commit().clone();
    let failed = crate::domain_computation::execution_runtime::product_world::preserve_delivery_authority(
        worth_proof::TransitionOutcome::Failed(
            worth_runtime_bridge::facade::BridgeCorrespondenceAdmissionFailure::SourceLoadFailed,
        ),
        performed,
    );
    let returned = failed
        .into_retry_change()
        .expect("a Bridge failure must return the exact performed-change authority");
    assert_eq!(returned.product_commit(), &commit);
    assert!(first_root.accepts_performed_change(&returned));

    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "committed", &request);
    let drift = admitted_program(&world, &principal, &account, &request, "different");
    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(drift, idempotency(9, 8))
    else {
        panic!("reusing a key for another intent must be denied");
    };
    assert_eq!(
        denial.kind(),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    );
}

#[test]
fn equivalent_retry_prepared_at_the_original_product_recovers_exact_owner_evidence() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let selected_commit = world.selected_product().product().selected_commit().clone();
    let first = admitted_program(&world, &principal, &account, &request, "committed");
    let stale_retry = admitted_program(&world, &principal, &account, &request, "committed");
    let binding = idempotency(93, 93);

    let WorthQueryApplicationCommitOutcome::Committed(committed) = world
        .application
        .compare_and_commit_application(first, binding)
    else {
        panic!("the first application must publish its exact owner evidence");
    };
    let current_commit = world.selected_product().product().selected_commit().clone();

    let outcome = world
        .application
        .compare_and_commit_application(stale_retry, binding);
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = outcome else {
        panic!("an equivalent retry must recover the exact completed owner evidence: {outcome:?}");
    };
    assert!(recovered.is_same_authoritative_commit(&committed));
    assert_eq!(
        recovered.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
    assert_ne!(selected_commit, current_commit);
}

#[test]
fn same_caller_intent_cannot_cross_installed_operation_identity() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let first = admitted_program(&world, &principal, &account, &request, "committed");
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(first, idempotency(10, 10)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));

    let account = resolved_account(&world, "committed", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(MultiTouchOperation::reference())
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
    let resolution = world
        .application
        .resolve_admitted_application_idempotency(&admission, idempotency(10, 10))
        .unwrap()
        .into_resolution();
    assert_eq!(
        resolution,
        WorthQueryApplicationIdempotencyResolution::IntentDrift
    );
}

#[test]
fn same_operation_intent_cannot_cross_an_admitted_scope() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let first_account = resolved_account(&world, "open", &request);
    let first = admitted_program(&world, &principal, &first_account, &request, "first-scope");

    assert!(matches!(
        world
            .application
            .compare_and_commit_application(first, idempotency(11, 11)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let principal = authenticated_principal(&world, &request);
    let second_account = resolved_account(&world, "unrelated", &request);
    let second = admitted_program(
        &world,
        &principal,
        &second_account,
        &request,
        "second-scope",
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(second, idempotency(11, 11))
    else {
        panic!("reusing one operation intent across scopes must deny");
    };
    assert_eq!(
        denial.kind(),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    );
}
