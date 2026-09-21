use tempfile::tempdir;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalEffectObligation, PhysicalExecutorCommand, PhysicalSchedulerDenial,
    PhysicalStoreCloseOutcome, PhysicalStoreClosePhase, PhysicalWorkEffectFate,
    PhysicalWorkExecutionOutcome, PhysicalWorkPreEffectDenial,
};
use worth_store_physical_backend::MediaOperationRole;
use worth_store_physical_backend::{FilesystemAccessPosture, MediaFaultDirective};

use super::{
    executor::admitted_write,
    fixture::{
        disjoint_artifact_mutation_fixture, overlapping_mutation_fixture,
        serving_from_initialization_with_work_profile, serving_from_open_with_work_profile,
        work_fixture,
    },
    policy_receipt,
    scheduler::{ready_work, secure_demand, write_demand},
};

#[test]
fn independent_mutation_capabilities_execute_without_a_global_runtime_borrow() {
    let root = tempdir().unwrap();
    let (profile, first_request, second_request) = disjoint_artifact_mutation_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::production(
        FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let first_gate = authority.pause_gate();
    let second_gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![
            authority.rule(
                MediaOperationRole::PositionedWrite,
                1,
                MediaFaultDirective::PauseBefore(first_gate.clone()),
            ),
            authority.rule(
                MediaOperationRole::PositionedWrite,
                2,
                MediaFaultDirective::PauseBefore(second_gate.clone()),
            ),
        ])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root.path()).unwrap(),
    )
    .unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("faulted media should admit"),
    };
    let (format, _, access) = super::super::configuration();
    let serving = super::super::success(open_record_store!(media, |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
            .with_physical_work_profile(profile)
    },));
    let before = serving.media_counters();
    let first = admitted_write(&serving, first_request);
    let second = admitted_write(&serving, second_request);
    assert_eq!(
        first
            .concurrency_scope()
            .relation(&second.concurrency_scope()),
        worth_store::physical_runtime::PhysicalWorkConcurrencyRelation::DisjointArtifacts,
        "C5_PREDICATE:branch-label-disjointness exact physical coordinates, not label-like digests, must determine concurrency",
    );
    let first = PhysicalExecutorCommand::exact_write(first, b"thread01".as_slice()).unwrap();
    let second = PhysicalExecutorCommand::exact_write(second, b"thread02".as_slice()).unwrap();
    let (first, second, overlapped) = std::thread::scope(|scope| {
        let first = scope.spawn(|| serving.execute_physical_work(first));
        let second = scope.spawn(|| serving.execute_physical_work(second));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while (first_gate.reached_context().is_none() || second_gate.reached_context().is_none())
            && std::time::Instant::now() < deadline
        {
            std::thread::yield_now();
        }
        let overlapped =
            first_gate.reached_context().is_some() && second_gate.reached_context().is_some();
        let first_context = first_gate.reached_context();
        let second_context = second_gate.reached_context();
        first_gate.release();
        second_gate.release();
        if let (Some(first_context), Some(second_context)) = (first_context, second_context) {
            assert_ne!(first_context.role_ordinal(), second_context.role_ordinal());
            assert_eq!(first_context.requested_bytes(), 8);
            assert_eq!(second_context.requested_bytes(), 8);
        }
        (
            first.join().unwrap().unwrap(),
            second.join().unwrap().unwrap(),
            overlapped,
        )
    });

    assert_completed_distinct(&first, &second);
    let first_effect = first.settled().effect_identity().unwrap();
    let second_effect = second.settled().effect_identity().unwrap();
    assert_eq!(first_effect.work(), first.settled().intent().identity());
    assert_eq!(second_effect.work(), second.settled().intent().identity());
    assert_ne!(
        first_effect.backend_operation(),
        second_effect.backend_operation()
    );
    assert!(
        overlapped,
        "C5_PREDICATE:global-mutation-lock both disjoint target effects must reach the backend while the other is paused"
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite)
            - before.attempts_for(MediaOperationRole::PositionedWrite),
        2
    );
    assert!(matches!(
        serving.close_plan().execute(),
        PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

#[test]
fn overlapping_writes_conflict_before_the_second_effect() {
    let root = tempdir().unwrap();
    let (profile, first_request, second_request) = overlapping_mutation_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_work_profile(root.path(), profile);
    let before = serving.media_counters();
    let first = admitted_write(&serving, first_request);
    let ready = ready_work(&serving, second_request);
    let demand = write_demand(&serving, ready);
    let requested_budget = demand.queue_work().requested_budget();
    let backend_requirement = demand.queue_work().backend_requirement();
    let backend = serving
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let demand = secure_demand(demand, &backend);
    match serving.admit_physical_scheduler_demand(
        demand,
        &backend,
        policy_receipt(requested_budget),
    ) {
        Err(PhysicalSchedulerDenial::EffectConflict) => {}
        Err(denial) => panic!("an overlapping write must be refused before its effect, got {denial:?}"),
        Ok(_) => panic!("an overlapping write must be refused before its effect"),
    }
    let command =
        PhysicalExecutorCommand::exact_write(first, b"overlap!".as_slice()).unwrap();
    serving.execute_physical_work(command).unwrap();
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite)
            - before.attempts_for(MediaOperationRole::PositionedWrite),
        1,
        "the conflicting write must not reach the backend"
    );
    assert!(matches!(
        serving.close_plan().execute(),
        PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

#[test]
fn close_waits_for_a_dispatched_execution_capability_before_disposal() {
    let root = tempdir().unwrap();
    let (profile, _, request) = work_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::production(
        FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority.rule(
            MediaOperationRole::PositionedWrite,
            1,
            MediaFaultDirective::PauseBefore(gate.clone()),
        )])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root.path()).unwrap(),
    )
    .unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("faulted media should admit"),
    };
    let (format, _, access) = super::super::configuration();
    let serving = super::super::success(open_record_store!(media, |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
            .with_physical_work_profile(profile)
    },));
    let command = PhysicalExecutorCommand::exact_write(
        admitted_write(&serving, request),
        b"closing!".as_slice(),
    )
    .unwrap();
    let execution = serving.physical_work_execution();
    let close = serving.close_plan();
    let progress = close.observation();

    let (settled, closed) = std::thread::scope(|scope| {
        let effect = scope.spawn(move || execution.execute_physical_work(command));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while gate.reached_context().is_none() && std::time::Instant::now() < deadline {
            std::thread::yield_now();
        }
        assert!(
            gate.reached_context().is_some(),
            "effect never reached pause gate"
        );
        let closing = scope.spawn(move || close.execute());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !progress.reached(PhysicalStoreClosePhase::AdmissionStopped)
            && std::time::Instant::now() < deadline
        {
            std::thread::yield_now();
        }
        assert!(progress.reached(PhysicalStoreClosePhase::AdmissionStopped));
        assert!(
            !progress.reached(PhysicalStoreClosePhase::DispatchSettlementComplete),
            "C5_PREDICATE:duplicate-work-registry: close used a registry disjoint from active execution"
        );
        assert!(
            !progress.reached(PhysicalStoreClosePhase::SignalDisposed),
            "C5_PREDICATE:duplicate-work-registry: Signal disposed before the canonical execution registry drained"
        );
        gate.release();
        (effect.join().unwrap().unwrap(), closing.join().unwrap())
    });

    assert_eq!(
        settled.settled().evidence().fate(),
        PhysicalWorkEffectFate::WriteCompleted
    );
    assert!(matches!(closed, PhysicalStoreCloseOutcome::Closed { .. }));
    assert!(progress.reached(PhysicalStoreClosePhase::MediaReleased));
}

#[test]
fn overlapping_exact_writes_are_refused_before_a_second_effect() {
    let root = tempdir().unwrap();
    let (profile, _, request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let before = serving.media_counters();
    let first = admitted_write(&serving, request.clone());
    let ready = ready_work(&serving, request);
    let demand = write_demand(&serving, ready);
    let requested_budget = demand.queue_work().requested_budget();
    let backend = serving
        .admit_physical_scheduler_capability(demand.queue_work().backend_requirement())
        .unwrap();
    let demand = secure_demand(demand, &backend);
    match serving.admit_physical_scheduler_demand(
        demand,
        &backend,
        policy_receipt(requested_budget),
    ) {
        Err(PhysicalSchedulerDenial::EffectConflict) => {}
        Err(denial) => panic!("the same catalog range must conflict, got {denial:?}"),
        Ok(_) => panic!("the same catalog range must conflict before a second effect"),
    }
    serving
        .execute_physical_work(
            PhysicalExecutorCommand::exact_write(first, b"winner01".as_slice()).unwrap(),
        )
        .unwrap();
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite)
            - before.attempts_for(MediaOperationRole::PositionedWrite),
        1
    );
    let catalog = std::fs::read(root.path().join("families/records/bootstrap.catalog")).unwrap();
    assert_eq!(&catalog[8..16], b"winner01");
    serving.close();
}

#[test]
fn cancellation_and_dispatch_have_one_atomic_physical_winner() {
    let root = tempdir().unwrap();
    let (profile, _, request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    for ordinal in 0..32_u8 {
        let admitted = admitted_write(&serving, request.clone());
        let consumer = admitted.consumer_handle();
        let command =
            PhysicalExecutorCommand::exact_write(admitted, [ordinal; 8].as_slice()).unwrap();
        let barrier = std::sync::Barrier::new(3);
        let before = serving.media_counters();
        let (execution, cancellation) = std::thread::scope(|scope| {
            let execute_barrier = &barrier;
            let cancel_barrier = &barrier;
            let execution = scope.spawn(|| {
                execute_barrier.wait();
                serving.execute_physical_work(command)
            });
            let cancellation = scope.spawn(|| {
                cancel_barrier.wait();
                serving.cancel_physical_work(consumer)
            });
            barrier.wait();
            (
                execution.join().unwrap(),
                cancellation.join().unwrap().unwrap(),
            )
        });
        let effects = serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite)
            - before.attempts_for(MediaOperationRole::PositionedWrite);
        match execution {
            Ok(settled) => {
                assert_eq!(
                    settled.settled().evidence().fate(),
                    PhysicalWorkEffectFate::WriteCompleted
                );
                assert_eq!(effects, 1);
                assert_eq!(
                    cancellation.obligation(),
                    PhysicalEffectObligation::SettlementContinues
                );
            }
            Err(PhysicalWorkPreEffectDenial::ConsumerCancelled) => {
                assert_eq!(effects, 0);
                assert_eq!(
                    cancellation.obligation(),
                    PhysicalEffectObligation::NotDispatched
                );
            }
            Err(other) => panic!("unexpected race denial: {other:?}"),
        }
    }
    assert!(matches!(
        serving.close_plan().execute(),
        PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

fn assert_completed_distinct(
    first: &PhysicalWorkExecutionOutcome,
    second: &PhysicalWorkExecutionOutcome,
) {
    assert_eq!(
        first.settled().evidence().fate(),
        PhysicalWorkEffectFate::WriteCompleted
    );
    assert_eq!(
        second.settled().evidence().fate(),
        PhysicalWorkEffectFate::WriteCompleted
    );
    assert_ne!(
        first.settled().intent().identity(),
        second.settled().intent().identity()
    );
}
