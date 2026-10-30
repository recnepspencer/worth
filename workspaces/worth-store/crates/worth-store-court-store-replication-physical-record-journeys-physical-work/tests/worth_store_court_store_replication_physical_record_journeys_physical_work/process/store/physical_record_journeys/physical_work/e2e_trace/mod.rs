#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::e2e_trace::*;
use tempfile::tempdir;

use {
    crate::physical_record_journeys::physical_work::fixture::io_pressure::disjoint_io_pressure_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::ready_read_work,
};

#[test]
fn joined_pressure_trace_covers_disjoint_io_blocking_scheduler_exhaustion_and_effects() {
    let root = tempdir().unwrap();
    let (
        profile,
        [first_read_request, second_read_request],
        [first_write_request, second_write_request],
    ) = disjoint_io_pressure_fixture();
    // Five works plus the queue slot withheld for checkpoint or retirement.
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(
        6,
        1,
        5,
        1024 * 1024,
        5 * 1024 * 1024,
    )
    .unwrap()
    .with_terminal_evidence_capacity(8)
    .unwrap();
    let profile = profile.with_capacity(capacity);
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::production(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let pause = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority.rule(
            worth_store_physical_backend::MediaOperationRole::PositionedWrite,
            1,
            worth_store_physical_backend::MediaFaultDirective::PauseBefore(pause.clone()),
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
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("faulted media should admit"),
    };
    let (format, _, access) = worth_store_testkit_store::physical_record_journeys::configuration();
    let serving = worth_store_testkit_store::physical_record_journeys::success(open_record_store!(
        media,
        |durability| {
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
                .with_physical_work_profile(profile)
        },
    ));
    let first_read = admitted_read(&serving, first_read_request.clone());
    let second_read = admitted_read(&serving, second_read_request);
    let first_write = admitted_write(&serving, first_write_request);
    let second_write = admitted_write(&serving, second_write_request);
    assert_eq!(
        first_read
            .concurrency_scope()
            .relation(&second_read.concurrency_scope()),
        worth_store::physical_runtime::PhysicalWorkConcurrencyRelation::DisjointArtifacts
    );
    assert_eq!(
        first_write
            .concurrency_scope()
            .relation(&second_write.concurrency_scope()),
        worth_store::physical_runtime::PhysicalWorkConcurrencyRelation::DisjointArtifacts
    );
    let before = serving.media_counters();
    let pressure = ready_read_work(&serving, first_read_request);
    let pressure_identity = pressure.intent().identity();
    assert_scheduler_breadth_exhaustion(&serving, pressure);
    assert_eq!(
        serving.media_counters(),
        before,
        "scheduler exhaustion must deny before any backend effect"
    );
    wait_until(|| {
        serving
            .physical_work_counters()
            .total(worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal)
            == 1
    });
    assert_queued_mix(serving.physical_work_counters());
    let observation = serving.physical_work_observer();
    let first_command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        first_write,
        b"blocked!".as_slice(),
    )
    .unwrap();
    let remainder = vec![
        worth_store::physical_runtime::PhysicalExecutorCommand::read(first_read).unwrap(),
        worth_store::physical_runtime::PhysicalExecutorCommand::read(second_read).unwrap(),
        worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
            second_write,
            b"follows!".as_slice(),
        )
        .unwrap(),
    ]
    .into_boxed_slice();
    let first_execution = serving.physical_work_execution();
    let remaining_execution = serving.physical_work_execution();

    let (first, remainder) = std::thread::scope(|scope| {
        let first = scope.spawn(move || first_execution.execute_physical_work(first_command));
        wait_until(|| pause.reached_context().is_some());
        let blocked = serving.physical_work_counters();
        assert_eq!(
            blocked.count(
                worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite,
                worth_store::physical_runtime::PhysicalWorkCounterStage::Dispatched,
            ),
            1
        );
        let remainder =
            scope.spawn(move || remaining_execution.execute_physical_work_batch(remainder));
        wait_until(|| {
            serving.physical_work_counters().count(
                worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead,
                worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
            ) == 3
        });
        pause.release();
        (first.join().unwrap().unwrap(), remainder.join().unwrap())
    });

    assert_eq!(
        first.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
    );
    assert!(remainder.denied_before_effect().is_empty());
    assert_eq!(remainder.executions().len(), 3);
    let final_counters = serving.physical_work_counters();
    assert_eq!(
        final_counters.count(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ),
        3
    );
    assert_eq!(
        final_counters.count(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ),
        2
    );
    assert_eq!(
        final_counters.total(worth_store::physical_runtime::PhysicalWorkCounterStage::Queued),
        0
    );
    assert_eq!(
        final_counters.total(worth_store::physical_runtime::PhysicalWorkCounterStage::Dispatched),
        0
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead),
        2
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before
                .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        2
    );
    let causal = observation.causal().records();
    assert_causal_trace(&causal);
    let closed = serving.close();
    assert_eq!(closed.work().declared(), 5);
    assert_eq!(closed.work().residual(), 0);
    assert_eq!(
        closed.work().drain().released_before_dispatch(),
        &[pressure_identity]
    );
}
