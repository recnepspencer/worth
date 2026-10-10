//! equivalent retry recovers original receipt while intent drift is denied.
use super::*;

#[test]
fn equivalent_retry_recovers_original_receipt_while_intent_drift_is_denied() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "committed")
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryApplicationCommitOutcome::Committed(mut original) =
        world.application.compare_and_commit_application(
            first,
            idempotency(9, 7),
            ExecutionAllocationPolicy::SystemAllocation,
        )
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

    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "committed", &request)
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "committed")
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(mut recovered) =
        world.application.compare_and_commit_application(
            retry,
            idempotency(9, 7),
            ExecutionAllocationPolicy::SystemAllocation,
        )
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
    assert!(
        !failed.is_retryable(),
        "Bridge admission failure is terminal"
    );
    let returned = failed
        .into_undelivered_change()
        .expect("a Bridge failure must return the exact performed-change authority");
    assert_eq!(returned.product_commit(), &commit);
    assert!(first_root.accepts_performed_change(&returned));
    let deferred =
        crate::domain_computation::execution_runtime::product_world::preserve_delivery_authority(
            worth_proof::TransitionOutcome::Deferred(
                worth_runtime_bridge::facade::BridgeCorrespondenceDeferred::GraphMutationInProgress,
            ),
            returned,
        );
    assert!(
        deferred.is_retryable(),
        "a busy Bridge may admit the same change later"
    );
    let returned = deferred
        .into_undelivered_change()
        .expect("deferral returns the performed-change authority");
    assert_eq!(returned.product_commit(), &commit);

    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "committed", &request)
        })
        .expect("the fixture read owns its installed host call");
    let drift = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "different")
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            drift,
            idempotency(9, 8),
            ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("reusing a key for another intent must be denied");
    };
    assert_eq!(
        denial.kind(),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    );
}
