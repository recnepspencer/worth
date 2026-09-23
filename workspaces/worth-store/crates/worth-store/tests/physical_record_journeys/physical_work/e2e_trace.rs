use tempfile::tempdir;
use worth_foundational::FoundationalPerformanceWorkClass;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalExecutorCommand, PhysicalRecordOpen,
    PhysicalRuntimeAdmission, PhysicalSchedulerDemand, PhysicalSchedulerDenial,
    PhysicalSignalSettlementOutcome, PhysicalStore, PhysicalWorkCapacity, PhysicalWorkCounterStage,
    PhysicalWorkEffectFate, PhysicalWorkOperationFamily, PhysicalWorkPressureClass,
    PhysicalWorkRecoveryDisposition,
};
use worth_store_io_scheduler::QueueExecutionAdmissionDenial;
use worth_store_physical_backend::{
    FilesystemAccessPosture, MediaFaultDirective, MediaOperationRole,
};

use super::{
    executor::admitted_write,
    fixture::{disjoint_io_pressure_fixture, serving_from_initialization_with_work_profile},
    scheduler::{exhausted_policy_receipt, policy_receipt_for, ready_read_work, secure_demand},
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
    let capacity = PhysicalWorkCapacity::new(6, 1, 5, 1024 * 1024, 5 * 1024 * 1024)
        .unwrap()
        .with_terminal_evidence_capacity(8)
        .unwrap();
    let profile = profile.with_capacity(capacity);
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let pause = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority.rule(
            MediaOperationRole::PositionedWrite,
            1,
            MediaFaultDirective::PauseBefore(pause.clone()),
        )])
        .unwrap();
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.path()).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("faulted media should admit"),
    };
    let (format, _, access) = super::super::configuration();
    let serving = super::super::success(open_record_store!(media, |durability| {
        PhysicalRecordOpen::new(format, access, durability).with_physical_work_profile(profile)
    },));
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
            .total(PhysicalWorkCounterStage::Terminal)
            == 1
    });
    assert_queued_mix(serving.physical_work_counters());
    let observation = serving.physical_work_observer();
    let first_command =
        PhysicalExecutorCommand::exact_write(first_write, b"blocked!".as_slice()).unwrap();
    let remainder = vec![
        PhysicalExecutorCommand::read(first_read).unwrap(),
        PhysicalExecutorCommand::read(second_read).unwrap(),
        PhysicalExecutorCommand::exact_write(second_write, b"follows!".as_slice()).unwrap(),
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
                PhysicalWorkOperationFamily::ArtifactRangeWrite,
                PhysicalWorkCounterStage::Dispatched,
            ),
            1
        );
        let remainder =
            scope.spawn(move || remaining_execution.execute_physical_work_batch(remainder));
        wait_until(|| {
            serving.physical_work_counters().count(
                PhysicalWorkOperationFamily::ArtifactRangeRead,
                PhysicalWorkCounterStage::Terminal,
            ) == 3
        });
        pause.release();
        (first.join().unwrap().unwrap(), remainder.join().unwrap())
    });

    assert_eq!(
        first.settled().evidence().fate(),
        PhysicalWorkEffectFate::WriteCompleted
    );
    assert!(remainder.denied_before_effect().is_empty());
    assert_eq!(remainder.executions().len(), 3);
    let final_counters = serving.physical_work_counters();
    assert_eq!(
        final_counters.count(
            PhysicalWorkOperationFamily::ArtifactRangeRead,
            PhysicalWorkCounterStage::Terminal,
        ),
        3
    );
    assert_eq!(
        final_counters.count(
            PhysicalWorkOperationFamily::ArtifactRangeWrite,
            PhysicalWorkCounterStage::Terminal,
        ),
        2
    );
    assert_eq!(final_counters.total(PhysicalWorkCounterStage::Queued), 0);
    assert_eq!(
        final_counters.total(PhysicalWorkCounterStage::Dispatched),
        0
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedRead)
            - before.attempts_for(MediaOperationRole::PositionedRead),
        2
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite)
            - before.attempts_for(MediaOperationRole::PositionedWrite),
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

fn assert_causal_trace(causal: &[worth_store::physical_runtime::PhysicalWorkCausalRecord]) {
    assert_eq!(causal.len(), 4);
    let mut terminal_totals = causal
        .iter()
        .map(|record| {
            assert!(record.backend_operation().is_some());
            assert_ne!(
                record.derived_completion(),
                Some(PhysicalSignalSettlementOutcome::DerivedStateUnavailable)
            );
            assert!(record.derived_completion().is_some());
            match record.effect_fate() {
                PhysicalWorkEffectFate::ReadCompleted => {
                    assert_eq!(record.recovery(), PhysicalWorkRecoveryDisposition::NoEffect);
                }
                PhysicalWorkEffectFate::WriteCompleted => {
                    assert_eq!(
                        record.recovery(),
                        PhysicalWorkRecoveryDisposition::ContinueSettlement
                    );
                }
                fate => panic!("joined trace observed an unexpected effect fate: {fate:?}"),
            }
            record.counters().total(PhysicalWorkCounterStage::Terminal)
        })
        .collect::<Vec<_>>();
    terminal_totals.sort_unstable();
    assert_eq!(terminal_totals, [2, 3, 4, 5]);
    for (index, record) in causal.iter().enumerate() {
        for other in &causal[index + 1..] {
            assert_ne!(record.identity(), other.identity());
            assert_ne!(record.backend_operation(), other.backend_operation());
            assert_ne!(record.signal_request(), other.signal_request());
        }
    }
}

fn assert_scheduler_breadth_exhaustion(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    ready: worth_store::physical_runtime::ReadyPhysicalWork,
) {
    let demand = PhysicalSchedulerDemand::foreground(
        ready,
        super::reserved_buffered_file_read(serving),
        None,
    )
    .unwrap();
    let work = demand.queue_work();
    let backend_requirement = work.backend_requirement();
    let requested_budget = work.requested_budget();
    let backend = serving
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let demand = secure_demand(demand, &backend);
    assert!(matches!(
        serving.admit_physical_scheduler_demand(
            demand,
            &backend,
            exhausted_policy_receipt(
                requested_budget,
                FoundationalPerformanceWorkClass::AuthoritativeRead,
            ),
        ),
        Err(PhysicalSchedulerDenial::Queue(
            QueueExecutionAdmissionDenial::PolicyReceiptBudgetMismatch {
                kind: worth_foundational::FoundationalPerformanceBudgetKind::Breadth,
                ..
            }
        ))
    ));
}

fn admitted_read(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: worth_store::physical_runtime::PhysicalReadWorkRequest,
) -> worth_store::physical_runtime::ResourceAdmittedPhysicalWork {
    let demand = PhysicalSchedulerDemand::foreground(
        ready_read_work(serving, request),
        super::reserved_buffered_file_read(serving),
        None,
    )
    .unwrap();
    let work = demand.queue_work();
    let backend_requirement = work.backend_requirement();
    let requested_budget = work.requested_budget();
    let backend = serving
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let demand = secure_demand(demand, &backend);
    serving
        .admit_physical_scheduler_demand(
            demand,
            &backend,
            policy_receipt_for(
                requested_budget,
                0,
                FoundationalPerformanceWorkClass::AuthoritativeRead,
            ),
        )
        .unwrap()
}

fn assert_queued_mix(counters: worth_store::physical_runtime::PhysicalWorkCounterSnapshot) {
    assert_eq!(counters.total(PhysicalWorkCounterStage::Queued), 4);
    assert_eq!(counters.total(PhysicalWorkCounterStage::Terminal), 1);
    assert_eq!(
        counters.count_under_pressure(
            PhysicalWorkOperationFamily::ArtifactRangeRead,
            PhysicalWorkPressureClass::ForegroundInternalRead,
            PhysicalWorkCounterStage::Queued,
        ),
        2
    );
    assert_eq!(
        counters.count_under_pressure(
            PhysicalWorkOperationFamily::ArtifactRangeWrite,
            PhysicalWorkPressureClass::ForegroundMutation,
            PhysicalWorkCounterStage::Queued,
        ),
        2
    );
}

fn wait_until(mut predicate: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !predicate() && std::time::Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(
        predicate(),
        "trace did not reach its bounded observation point"
    );
}
