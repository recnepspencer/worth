//! Genuine owner admission and original close-task metadata retention.
use super::*;
use crate::lifecycle::{RuntimeWorldOwnerLifecycleObservation, RuntimeWorldOwnerRoot};
use worth_execution::{CancellationToken, SerialMemoryBudget, SerialRequest, WorkCeilingDenial};
use worth_relational::facade::history::{BranchId, CommitId, RelationalCommitReceipt};

struct Clock;
impl crate::lifecycle::RuntimeWorldClockSource for Clock {
    fn now(&self) -> crate::lifecycle::RuntimeWorldInstant {
        crate::lifecycle::RuntimeWorldInstant::from_ticks(0)
    }
}
fn owner() -> (
    crate::branch::reference_test_fixture::RealReferenceFixture,
    RuntimeWorldOwnerRoot<(), (), (), (), ()>,
) {
    let mut fixture = crate::branch::reference_test_fixture::real_fixture(8, 8);
    let owner = RuntimeWorldOwnerRoot::new(fixture.owner_inputs(
        crate::budget::RuntimeWorldBudgets::without_policy_limits(),
        crate::lifecycle::RuntimeWorldClock::from_source(Clock),
    ))
    .unwrap();
    (fixture, owner)
}

#[test]
fn preclose_observation_uses_original_gates_without_allocating_a_close_task() {
    let (_fixture, owner) = owner();
    let memory = SerialMemoryBudget::new(Some(0));
    let policy = SerialRequest::from_memory(memory.clone(), CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&policy);
    owner.try_preflight_close_admission(&request).unwrap();
    assert_eq!(memory.charged_memory_bytes(), 0);
    assert!(owner.state.close.lock().unwrap().progress.is_none());
    assert!(owner.state.bootstrap.try_lock().is_ok());
    assert!(owner.state.operation.state.try_lock().is_ok());
    assert!(matches!(
        owner.try_prepare_close_admission(&request),
        Err(Denial::Execution(WorkCeilingDenial::Admission(_)))
    ));
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open
    );
    assert!(owner.state.close.lock().unwrap().progress.is_none());
    assert_eq!(memory.charged_memory_bytes(), 0);
}

#[test]
fn prepared_close_task_retains_original_metadata_grant_until_owner_release() {
    let (_fixture, owner) = owner();
    let memory = SerialMemoryBudget::new(Some(1 << 20));
    let policy = SerialRequest::from_memory(memory.clone(), CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&policy);
    owner.try_preflight_close_admission(&request).unwrap();
    assert_eq!(memory.charged_memory_bytes(), 0);
    let admission = owner.try_prepare_close_admission(&request).unwrap();
    assert!(memory.charged_memory_bytes() > 0);
    drop(admission);
    assert_eq!(memory.charged_memory_bytes(), 0);
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open
    );

    let admission = owner.try_prepare_close_admission(&request).unwrap();
    let task = admission.task.clone();
    let retained = memory.charged_memory_bytes();
    admission.begin();
    assert!(Arc::ptr_eq(
        owner.state.close.lock().unwrap().progress.as_ref().unwrap(),
        &task
    ));
    assert_eq!(memory.charged_memory_bytes(), retained);
    drop(owner);
    assert_eq!(memory.charged_memory_bytes(), retained);
    drop(task);
    assert_eq!(memory.charged_memory_bytes(), 0);
}

#[test]
fn recovered_receipt_storage_refuses_short_grants_and_stays_funded_until_drop() {
    // Descriptive data tests copying only; it issues no Native authority.
    let original = RelationalCommitReceipt {
        commit_id: CommitId(7),
        version_id: worth_relational::facade::identity::VersionId(8),
        branch_id: BranchId("original-recovered-branch".into()),
        parents: vec![CommitId(3), CommitId(5)],
    };
    let needed = original.branch_id.0.len() as u64 + 2 * std::mem::size_of::<CommitId>() as u64;
    // The successful original grant also covers ExecutionRequest's
    // transient scope storage. Only the receipt's exact owned bytes may
    // remain charged after the copy returns.
    for budget in [needed - 1, needed + 64 * 1024] {
        let memory = SerialMemoryBudget::new(Some(budget));
        let policy = SerialRequest::from_memory(memory.clone(), CancellationToken::new(), None);
        let request = ExecutionRequest::serial(&policy);
        let copied = RecoveredCloseReceipt::copy_for_request(&request, &original);
        if budget < needed {
            assert!(matches!(
                copied,
                Err(Denial::Execution(
                    worth_execution::WorkCeilingDenial::Admission(_)
                ))
            ));
            assert_eq!(memory.charged_memory_bytes(), 0);
        } else {
            let copied = copied.unwrap();
            assert_eq!(copied.receipt(), &original);
            assert_eq!(memory.charged_memory_bytes(), needed);
            drop(copied);
            assert_eq!(memory.charged_memory_bytes(), 0);
        }
    }
}

#[test]
fn recovered_receipt_copy_observes_the_original_work_stop_before_allocating() {
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(1 << 20),
            Some(0),
        ),
    );
    let policy = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&policy);
    let original = RelationalCommitReceipt {
        commit_id: CommitId(7),
        version_id: worth_relational::facade::identity::VersionId(8),
        branch_id: BranchId("original-recovered-branch".into()),
        parents: vec![CommitId(3), CommitId(5)],
    };
    assert!(matches!(
        RecoveredCloseReceipt::copy_for_request(&request, &original),
        Err(Denial::Stopped(worth_execution::MapKernelStop::WorkCeiling))
    ));
    assert_eq!(policy.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn closed_home_retirement_preserves_shared_root_history_on_busy_and_stop_then_retires_without_drop()
{
    let (fixture, root) = owner();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    assert!(matches!(
        root.bootstrap_root(fixture.bootstrap_intent()),
        crate::branch::RuntimeWorldBootstrapOutcome::Performed(_)
    ));
    let world = crate::lifecycle::RuntimeWorldOwner {
        transferred_to_home_retirement: false,
        root: Arc::new(root),
    };
    let weak = Arc::downgrade(&world.root);
    let report = world.root.close(&request).unwrap();
    let mut cleanup = world.result_snapshot_cleanup_after_close(&request).unwrap();
    loop {
        match cleanup
            .advance(&request, fixture.relational_runtime())
            .unwrap()
        {
            crate::lifecycle::RuntimeWorldResultSnapshotCleanupProgress::Advanced => {}
            crate::lifecycle::RuntimeWorldResultSnapshotCleanupProgress::Complete(_) => break,
        }
    }
    let counters = world.root.state.retention.cost_snapshot();
    assert!(
        world
            .root
            .state
            .retention
            .active_component_obligation_count()
            >= 4,
        "the real root head and history still share their original owner leases"
    );
    let task = world
        .root
        .state
        .close
        .lock()
        .unwrap()
        .retirement_task()
        .unwrap();
    let native = fixture.relational_runtime().branch_basis_cost_counters();
    let mut original = world
        .into_home_retirement(&report, cleanup)
        .unwrap_or_else(|_| panic!("same actual closed owner"));
    let guard = task.lock().unwrap();
    assert!(matches!(
        original.advance(&request, fixture.relational_runtime()),
        Err(crate::lifecycle::RuntimeWorldHomeRetirementDenial::Close(
            Denial::Busy
        ))
    ));
    drop(guard);
    assert_eq!(
        weak.upgrade().unwrap().state.retention.cost_snapshot(),
        counters
    );
    assert_eq!(
        fixture.relational_runtime().branch_basis_cost_counters(),
        native
    );
    let cancellation = worth_execution::CancellationSource::new();
    cancellation.cancel();
    let stopped = serial.clone().with_cancellation(cancellation.token());
    assert!(original
        .advance(
            &ExecutionRequest::serial(&stopped),
            fixture.relational_runtime()
        )
        .is_err());
    assert_eq!(
        weak.upgrade().unwrap().state.retention.cost_snapshot(),
        counters
    );
    loop {
        match original
            .advance(&request, fixture.relational_runtime())
            .unwrap()
        {
            crate::lifecycle::RuntimeWorldHomeRetirementProgress::Advanced => {}
            crate::lifecycle::RuntimeWorldHomeRetirementProgress::Complete => break,
        }
    }
    let retired = original.into_retired().unwrap();
    let root = weak
        .upgrade()
        .expect("same actual World allocation stays alive");
    assert_eq!(root.state.retention.active_component_obligation_count(), 0);
    assert_eq!(
        root.state.retention.cost_snapshot().owner_drop_releases(),
        counters.owner_drop_releases()
    );
    assert_eq!(
        fixture
            .relational_runtime()
            .branch_basis_cost_counters()
            .external_retention_drop_releases,
        native.external_retention_drop_releases
    );
    drop(root);
    retired.reclaim();
    assert!(
        weak.upgrade().is_none(),
        "actual World destruction belongs to explicit unbudgeted reclaim"
    );
}
