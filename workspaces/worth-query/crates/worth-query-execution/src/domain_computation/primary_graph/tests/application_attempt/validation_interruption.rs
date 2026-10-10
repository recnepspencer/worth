//! Real Query publication cancellation after native transaction staging.
use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationCommitOutcome,
};
use crate::domain_computation::WorthQueryInvariantExecutionDenialKind;
use std::time::{Duration, Instant};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_relational::facade::mvcc::RelationalOperationInterruption;

#[test]
fn omitted_work_budget_keeps_live_cancellation_through_native_validation() {
    let world = installed_authorization_world(true);
    let installed = world
        .application
        .installed_schema()
        .installed_operation(super::TouchAccountOperation::reference())
        .unwrap();
    assert_eq!(installed.contracts().execution_strategy().unwrap().envelope()
        .optional_scale_ceiling(worth_query_declaration::facade::domain_computation::WorthQuerySemanticScaleAxis::WorkItems), None);
    let source = WorthQueryCancellationSource::new();
    let request =
        WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(30), source.token());
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
    let rejected = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "after-staging",
            )
        })
        .expect("the fixture read owns its installed host call");
    // This observer cancels the actual admitted request after push_batch succeeds.
    // It neither injects a failure nor bypasses the provider's validator.
    world.faults.cancel_next_staged_validation(source);
    let outcome = world.application.compare_and_commit_application(
        rejected,
        idempotency(94, 94),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("post-staging cancellation must deny the real publication: {outcome:?}");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(
        denial.invariant_execution_failure().unwrap().kind(),
        WorthQueryInvariantExecutionDenialKind::RequestInterrupted(
            RelationalOperationInterruption::Cancelled
        )
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0
    );
    let fresh = live_scope();
    let unchanged = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &fresh)
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(unchanged.entity_id(), account.entity_id());
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &fresh)
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(
                &phase,
                &world,
                &principal,
                &unchanged,
                &fresh,
                "after-staging",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(
        matches!(
            world.application.compare_and_commit_application(
                retry,
                idempotency(94, 94),
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
            ),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ),
        "cancelled attempt must not consume its key"
    );
    let landed = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(
                &(phase).test_host_request(),
                &world,
                "after-staging",
                &live_scope(),
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(landed.entity_id(), account.entity_id());
}
