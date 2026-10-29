pub(crate) use {tempfile::tempdir};
pub(crate) use {worth_foundational::FoundationalPerformanceWorkClass};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalExecutorCommand, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalSchedulerDemand, worth_store::physical_runtime::PhysicalSchedulerDenial, worth_store::physical_runtime::PhysicalSignalSettlementOutcome, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWorkCapacity, worth_store::physical_runtime::PhysicalWorkCounterStage, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkPressureClass, worth_store::physical_runtime::PhysicalWorkRecoveryDisposition};
pub(crate) use {worth_store_io_scheduler::QueueExecutionAdmissionDenial};
pub(crate) use {worth_store_physical_backend::FilesystemAccessPosture, worth_store_physical_backend::MediaFaultDirective, worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write, crate::physical_record_journeys::physical_work::fixture::io_pressure::disjoint_io_pressure_fixture, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_work::scheduler::policy_evidence::exhausted_policy_receipt, worth_store_testkit_store::physical_work::scheduler::policy_evidence::policy_receipt_for, worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::ready_read_work, worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::secure_demand};

pub(crate) fn assert_causal_trace(causal: &[worth_store::physical_runtime::PhysicalWorkCausalRecord]) {
    assert_eq!(causal.len(), 4);
    let mut terminal_totals = causal
        .iter()
        .map(|record| {
            assert!(record.backend_operation().is_some());
            assert_ne!(
                record.derived_completion(),
                Some(worth_store::physical_runtime::PhysicalSignalSettlementOutcome::DerivedStateUnavailable)
            );
            assert!(record.derived_completion().is_some());
            match record.effect_fate() {
                worth_store::physical_runtime::PhysicalWorkEffectFate::ReadCompleted => {
                    assert_eq!(record.recovery(), worth_store::physical_runtime::PhysicalWorkRecoveryDisposition::NoEffect);
                }
                worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted => {
                    assert_eq!(
                        record.recovery(),
                        worth_store::physical_runtime::PhysicalWorkRecoveryDisposition::ContinueSettlement
                    );
                }
                fate => panic!("joined trace observed an unexpected effect fate: {fate:?}"),
            }
            record.counters().total(worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal)
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

pub(crate) fn assert_scheduler_breadth_exhaustion(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    ready: worth_store::physical_runtime::ReadyPhysicalWork,
) {
    let demand = worth_store::physical_runtime::PhysicalSchedulerDemand::foreground(
        ready,
        worth_store_testkit_store::physical_work::scheduler_reservation::reserved_buffered_file_read(serving),
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
                worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeRead,
            ),
        ),
        Err(worth_store::physical_runtime::PhysicalSchedulerDenial::Queue(
            worth_store_io_scheduler::QueueExecutionAdmissionDenial::PolicyReceiptBudgetMismatch {
                kind: worth_foundational::FoundationalPerformanceBudgetKind::Breadth,
                ..
            }
        ))
    ));
}

pub(crate) fn admitted_read(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: worth_store::physical_runtime::PhysicalReadWorkRequest,
) -> worth_store::physical_runtime::ResourceAdmittedPhysicalWork {
    let demand = worth_store::physical_runtime::PhysicalSchedulerDemand::foreground(
        ready_read_work(serving, request),
        worth_store_testkit_store::physical_work::scheduler_reservation::reserved_buffered_file_read(serving),
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
                worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeRead,
            ),
        )
        .unwrap()
}

pub(crate) fn assert_queued_mix(counters: worth_store::physical_runtime::PhysicalWorkCounterSnapshot) {
    assert_eq!(counters.total(worth_store::physical_runtime::PhysicalWorkCounterStage::Queued), 4);
    assert_eq!(counters.total(worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal), 1);
    assert_eq!(
        counters.count_under_pressure(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead,
            worth_store::physical_runtime::PhysicalWorkPressureClass::ForegroundInternalRead,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Queued,
        ),
        2
    );
    assert_eq!(
        counters.count_under_pressure(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite,
            worth_store::physical_runtime::PhysicalWorkPressureClass::ForegroundMutation,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Queued,
        ),
        2
    );
}

pub(crate) fn wait_until(mut predicate: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !predicate() && std::time::Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(
        predicate(),
        "trace did not reach its bounded observation point"
    );
}