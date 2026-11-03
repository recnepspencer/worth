//! Original-request admission and retained partial head retirement.
use super::product_cas_loss::resolve_one_race;
use crate::lifecycle::{
    RuntimeWorldCloseDenial as Denial, RuntimeWorldOwnerLifecycleObservation as Phase,
};
use crate::publication::CompositeLateCancellationPosture;
use worth_execution::{CancellationSource, ExecutionRequest};

#[test]
fn memory_close_refuses_actual_unresolved_business_custody() {
    let policy = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&policy);
    let race = resolve_one_race(&request, CompositeLateCancellationPosture::NotRequested);
    assert_eq!(race.owner.recovery_record_count(), 1);
    let delivered = race
        .owner
        .recover_performed_publication(race.winner_head.selected_commit())
        .unwrap();
    drop(
        delivered
            .expect("original dropped winner delivery")
            .consume(),
    );
    let before = race.retained.live_obligation_count();
    assert!(matches!(
        race.owner.close(&request),
        Err(Denial::UnresolvedBusinessCustody)
    ));
    assert_eq!(race.owner.lifecycle_observation(), Phase::Open);
    assert_eq!(race.owner.recovery_record_count(), 1);
    assert_eq!(race.retained.live_obligation_count(), before);
}

#[test]
fn close_stops_before_seal_and_does_not_wait_for_admission_locks() {
    let (_fixture, owner, _head) = super::publication::setup();
    let cancellation = CancellationSource::new();
    let policy =
        crate::test_execution_policy::serial_request().with_cancellation(cancellation.token());
    let request = ExecutionRequest::serial(&policy);
    let ledger = owner.state.operation.state.lock().unwrap();
    assert!(matches!(owner.close(&request), Err(Denial::Busy)));
    drop(ledger);
    assert_eq!(owner.lifecycle_observation(), Phase::Open);
    cancellation.cancel();
    assert!(matches!(
        owner.close(&request),
        Err(Denial::Stopped(worth_execution::MapKernelStop::Cancelled))
    ));
    assert_eq!(owner.lifecycle_observation(), Phase::Open);
    let retry = crate::test_execution_policy::serial_request();
    assert!(owner.close(&ExecutionRequest::serial(&retry)).is_ok());
    assert_eq!(owner.lifecycle_observation(), Phase::Closed);
}

fn reused_children() -> (
    crate::branch::reference_test_fixture::RealReferenceFixture,
    super::publication::TestOwner,
    crate::branch::ProductBranchObservation,
) {
    use crate::branch::*;
    use crate::lifecycle::*;
    let mut fixture = crate::branch::reference_test_fixture::real_fixture(8, 8);
    let owner = RuntimeWorldOwnerRoot::new(fixture.owner_inputs(
        crate::budget::RuntimeWorldBudgets::without_policy_limits(),
        RuntimeWorldClock::from_source(super::FixedClock),
    ))
    .unwrap();
    let root = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        RuntimeWorldBootstrapOutcome::Performed(value) => value.product_branch().clone(),
        other => panic!("actual bootstrap refused: {other:?}"),
    };
    let mut last = None;
    for name in ["alpha", "bravo"] {
        let policy = crate::test_execution_policy::serial_request();
        let request = ExecutionRequest::serial(&policy);
        let cancel = crate::publication::RuntimeWorldCancellationSource::new();
        let intent = ProductBranchCreationIntent::from_source(
            name,
            ProductBranchCreationPlans::new(
                RelationalBranchCreationPlan::ReuseExact,
                SignalBranchCreationPlan::ReuseExact,
            ),
        )
        .unwrap();
        match RuntimeWorldBranchService::create_product_branch(
            &owner,
            RuntimeWorldBranchCreationRequest::new(&request, root.clone(), intent, &cancel.token()),
        )
        .unwrap()
        {
            RuntimeWorldBranchCreationOutcome::Performed(head) => last = Some(head),
            other => panic!("actual exact reuse refused: {other:?}"),
        }
    }
    (fixture, owner, last.unwrap())
}

#[test]
fn close_retains_extracted_head_and_cursor_across_busy_stop_and_retry() {
    let (_fixture, owner, child) = reused_children();
    let public_owner = crate::lifecycle::RuntimeWorldOwner {
        transferred_to_home_retirement: false,
        root: std::sync::Arc::new(owner),
    };
    let owner = &public_owner.root;
    let lifecycle = public_owner.lifecycle_port();
    let observation = public_owner.observation_port();
    let cell = owner
        .state
        .branches
        .branch_cell(child.branch_identity())
        .unwrap();
    let held = cell.hold_for_test();
    let before = owner.state.retention.active_component_obligation_count();
    let releases = owner.state.history.counters().direct_protection_releases();
    let cancellation = CancellationSource::new();
    let policy =
        crate::test_execution_policy::serial_request().with_cancellation(cancellation.token());
    let request = ExecutionRequest::serial(&policy);
    assert!(matches!(
        lifecycle.close(&request, &public_owner),
        Err(crate::lifecycle::RuntimeWorldServiceDenial::Denied(
            Denial::Busy
        ))
    ));
    assert!(observation
        .observe_product_branch(child.branch_identity())
        .is_err());
    assert_eq!(owner.lifecycle_observation(), Phase::Closing);
    assert_eq!(
        owner.state.branches.branch_count(),
        1,
        "root only; bravo is retained by close"
    );
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        before - 2
    );
    assert_eq!(
        owner.state.history.counters().direct_protection_releases(),
        releases + 1
    );
    cancellation.cancel();
    assert!(matches!(
        lifecycle.close(&request, &public_owner),
        Err(crate::lifecycle::RuntimeWorldServiceDenial::Denied(
            Denial::Stopped(worth_execution::MapKernelStop::Cancelled)
        ))
    ));
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        before - 2
    );
    drop(held);
    let retry = crate::test_execution_policy::serial_request();
    let report = lifecycle
        .close(&ExecutionRequest::serial(&retry), &public_owner)
        .unwrap();
    assert_eq!(report.released_product_head_pins(), 2);
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        before - 4
    );
    assert_eq!(
        owner.state.history.counters().direct_protection_releases(),
        releases + 2
    );
    assert_eq!(owner.lifecycle_observation(), Phase::Closed);
    assert!(matches!(
        lifecycle.close(&request, &public_owner),
        Err(crate::lifecycle::RuntimeWorldServiceDenial::Denied(
            Denial::AlreadyClosed
        ))
    ));
}
