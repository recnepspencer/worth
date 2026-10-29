#[allow(unused_imports)] use crate::physical_record_journeys::physical_work::capacity::*;
use {tempfile::tempdir};
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::PhysicalEffectObligation, worth_store::physical_runtime::PhysicalExecutorCommand, worth_store::physical_runtime::PhysicalStoreCloseOutcome, worth_store::physical_runtime::PhysicalWorkCapacity, worth_store::physical_runtime::PhysicalWorkCapacityDimension, worth_store::physical_runtime::PhysicalWorkReadiness, worth_store::physical_runtime::PhysicalWorkSubmissionOutcome};
use {worth_store_physical_backend::MediaOperationRole};

use {worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::family_locality_fixture, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};
use {worth_store_testkit_store::physical_work::scheduler::policy_evidence::policy_receipt, worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::ready_work, worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::secure_demand, worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::write_demand};

#[test]
fn bounded_command_arena_defers_without_retaining_an_unadmitted_identity() {
    let root = tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(4, 1, 1_024, 1024 * 1024, 4 * 1024 * 1024).unwrap();
    let serving =
        serving_from_initialization_with_work_profile(root.path(), profile.with_capacity(capacity));
    let submission = serving.physical_read_submission();
    for _ in 0..4 {
        success(submission.submit(request.clone()));
    }
    assert!(matches!(
        submission.submit(request).into_raw(),
        worth_proof::TransitionOutcome::Deferred(deferred)
            if deferred.capacity() == 4
                && deferred.dimension() == worth_store::physical_runtime::PhysicalWorkCapacityDimension::Commands
    ));
    let closed = serving.close();
    assert_eq!(closed.work().declared(), 4);
    assert_eq!(closed.work().terminal().len(), 4);
    assert_eq!(closed.work().residual(), 0);
}

#[test]
fn completed_pre_effect_work_releases_capacity_for_sustained_churn() {
    let root = tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(1, 1, 1, 1024 * 1024, 1024 * 1024)
        .unwrap()
        .with_terminal_evidence_capacity(4)
        .unwrap();
    let serving =
        serving_from_initialization_with_work_profile(root.path(), profile.with_capacity(capacity));
    let submission = serving.physical_read_submission();

    for _ in 0..32 {
        let receipt = success(submission.submit(request.clone()));
        let admitted = serving.admit_physical_work(receipt).unwrap();
        match serving.request_physical_work(admitted).unwrap() {
            worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => {
                let cancellation = serving
                    .cancel_physical_work(ready.consumer_handle())
                    .unwrap();
                assert_eq!(
                    cancellation.obligation(),
                    worth_store::physical_runtime::PhysicalEffectObligation::NotDispatched
                );
                drop(ready);
            }
            worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(blocked) => {
                panic!(
                    "clean dependency unexpectedly blocked: {:?}",
                    blocked.condition()
                )
            }
        }
    }

    let closed = serving.close_plan().execute();
    assert!(matches!(closed, worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }));
    assert_eq!(closed.shutdown().work().declared(), 32);
    assert_eq!(
        closed
            .shutdown()
            .work()
            .drain()
            .cancelled_before_dispatch()
            .len(),
        4
    );
    assert_eq!(closed.shutdown().work().drain().evidence_capacity(), 4);
    assert_eq!(closed.shutdown().work().drain().evidence_overflow(), 0);
    assert_eq!(closed.shutdown().work().drain().safe_evidence_elided(), 28);
    assert_eq!(closed.shutdown().work().drain().exact_identity_count(), 32);
}

#[test]
fn successful_media_settlement_releases_capacity_for_sustained_churn() {
    let root = tempdir().unwrap();
    let (profile, _, mutation) = work_fixture();
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(1, 1, 1, 1024 * 1024, 1024 * 1024)
        .unwrap()
        .with_terminal_evidence_capacity(4)
        .unwrap();
    let serving =
        serving_from_initialization_with_work_profile(root.path(), profile.with_capacity(capacity));
    let before = serving.media_counters();

    for _ in 0..16 {
        let demand = write_demand(&serving, ready_work(&serving, mutation.clone()));
        let queue = demand.queue_work();
        let backend_requirement = queue.backend_requirement();
        let requested_budget = queue.requested_budget();
        let backend = serving
            .admit_physical_scheduler_capability(backend_requirement)
            .unwrap();
        let demand = secure_demand(demand, &backend);
        let admitted = serving
            .admit_physical_scheduler_demand(demand, &backend, policy_receipt(requested_budget))
            .unwrap();
        let command =
            worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(admitted, b"settled!".as_slice()).unwrap();
        serving.execute_physical_work(command).unwrap();
    }

    let after = serving.media_counters();
    assert_eq!(
        after.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        16
    );
    let closed = serving.close_plan().execute();
    assert!(matches!(closed, worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }));
    assert_eq!(closed.shutdown().work().declared(), 16);
    assert_eq!(closed.shutdown().work().residual(), 0);
}

#[test]
fn close_reconciles_a_dropped_pre_dispatch_signal_consumer() {
    let root = tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let receipt = success(serving.physical_read_submission().submit(request));
    let admitted = serving.admit_physical_work(receipt).unwrap();
    let ready = match serving.request_physical_work(admitted).unwrap() {
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => ready,
        worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(_) => panic!("dependency should be ready"),
    };

    drop(ready);

    let closed = serving.close_plan().execute();
    assert!(matches!(closed, worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }));
    assert_eq!(
        closed
            .shutdown()
            .signal_summary()
            .unwrap()
            .active_in_flight_node_count(),
        0
    );
}

#[test]
fn dropped_ready_work_releases_signal_and_command_capacity_one_before_close() {
    let root = tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(1, 1, 1, 1024 * 1024, 1024 * 1024)
        .unwrap()
        .with_terminal_evidence_capacity(16)
        .unwrap();
    let serving =
        serving_from_initialization_with_work_profile(root.path(), profile.with_capacity(capacity));
    let before = serving.media_counters();

    for _ in 0..8 {
        let receipt = success(serving.physical_read_submission().submit(request.clone()));
        let admitted = serving.admit_physical_work(receipt).unwrap();
        let ready = match serving.request_physical_work(admitted).unwrap_or_else(|denial| {
            panic!(
                "C5_PREDICATE:store-local-async-registry: local policy state overrode Signal admission: {denial:?}"
            )
        }) {
            worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => ready,
            worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(_) => panic!("clean dependency should be ready"),
        };
        drop(ready);
        await_signal_cleanup(&serving);
    }

    assert_eq!(serving.media_counters(), before);
    let closed = serving.close_plan().execute();
    assert!(matches!(closed, worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }));
    assert_eq!(closed.shutdown().work().declared(), 8);
    assert_eq!(closed.shutdown().work().residual(), 0);
}

#[test]
fn dropped_blocked_revalidation_releases_its_minimum_two_lineage_capacity() {
    let root = tempdir().unwrap();
    let (profile, request, _, delta) = family_locality_fixture();
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(2, 1, 2, 1024 * 1024, 2 * 1024 * 1024)
        .unwrap()
        .with_terminal_evidence_capacity(16)
        .unwrap();
    let serving =
        serving_from_initialization_with_work_profile(root.path(), profile.with_capacity(capacity));
    let before = serving.media_counters();

    for _ in 0..4 {
        let older = success(serving.physical_read_submission().submit(request.clone()));
        let newer = success(serving.physical_read_submission().submit(request.clone()));
        let older = serving.admit_physical_work(older).unwrap();
        let newer = serving.admit_physical_work(newer).unwrap();
        let older = match serving.request_physical_work(older).unwrap() {
            worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => ready,
            worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(_) => panic!("older request should initially be ready"),
        };
        let newer = match serving.request_physical_work(newer).unwrap() {
            worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => ready,
            worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(_) => panic!("newer request should be ready"),
        };
        serving
            .certification_apply_physical_aspect_delta(delta.clone())
            .unwrap();
        let blocked = match serving.revalidate_physical_work(older).unwrap() {
            worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(blocked) => blocked,
            worth_store::physical_runtime::PhysicalWorkReadiness::Ready(_) => {
                panic!("superseded active lineage should remain blocked")
            }
        };
        drop(newer);
        drop(blocked);
        await_signal_cleanup(&serving);
    }

    assert_eq!(serving.media_counters(), before);
    let closed = serving.close_plan().execute();
    assert!(matches!(closed, worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }));
    assert_eq!(closed.shutdown().work().declared(), 8);
    assert_eq!(closed.shutdown().work().residual(), 0);
}

#[test]
fn aggregate_scope_and_per_work_semantic_budgets_defer_before_retention() {
    let root = tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let scope_limited = worth_store::physical_runtime::PhysicalWorkCapacity::new(4, 1, 1, 1024 * 1024, 4 * 1024 * 1024).unwrap();
    let serving = serving_from_initialization_with_work_profile(
        root.path(),
        profile.clone().with_capacity(scope_limited),
    );
    let submission = serving.physical_read_submission();
    success(submission.submit(request.clone()));
    assert!(matches!(
        submission.submit(request.clone()).into_raw(),
        worth_proof::TransitionOutcome::Deferred(deferred)
            if deferred.dimension() == worth_store::physical_runtime::PhysicalWorkCapacityDimension::TotalScopeMembers
    ));
    serving.close();

    let semantic_limited = worth_store::physical_runtime::PhysicalWorkCapacity::new(4, 1, 1_024, 1, 4).unwrap();
    let reopened = worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile(
        root.path(),
        profile.with_capacity(semantic_limited),
    );
    assert!(matches!(
        reopened.physical_read_submission().submit(request).into_raw(),
        worth_proof::TransitionOutcome::Deferred(deferred)
            if deferred.dimension() == worth_store::physical_runtime::PhysicalWorkCapacityDimension::SemanticBytesPerWork
    ));
    assert_eq!(reopened.close().work().declared(), 0);
}
