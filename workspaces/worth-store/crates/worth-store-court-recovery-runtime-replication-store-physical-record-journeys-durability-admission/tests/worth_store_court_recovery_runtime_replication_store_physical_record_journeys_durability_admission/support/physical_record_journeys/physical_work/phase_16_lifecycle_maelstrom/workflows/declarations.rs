pub(crate) use {worth_foundational::FoundationalPerformanceWorkClass};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::PhysicalExecutorCommand, worth_store::physical_runtime::PhysicalMutationWorkRequest, worth_store::physical_runtime::PhysicalReadWorkRequest, worth_store::physical_runtime::PhysicalSchedulerDemand, worth_store::physical_runtime::PhysicalSchedulerDenial, worth_store::physical_runtime::PhysicalSignalSettlementOutcome, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::PhysicalWorkReadiness, worth_store::physical_runtime::PhysicalWorkRecoveryDisposition, worth_store::physical_runtime::PhysicalWorkRetryScheduleOutcome, worth_store::physical_runtime::PhysicalWritebackExecution, worth_store::physical_runtime::ReadyPhysicalWork, worth_store::physical_runtime::ResourceAdmittedPhysicalWork, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_io_scheduler::QueueExecutionAdmissionDenial};
pub(crate) use {worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement};
pub(crate) use {worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordFrameCoordinate};

pub(crate) fn ready_read(
    serving: &ServingPhysicalRuntime,
    request: PhysicalReadWorkRequest,
) -> ReadyPhysicalWork {
    let receipt = match serving
        .physical_read_submission()
        .submit(request)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        outcome => panic!("Phase 16 read submission should succeed: {outcome:?}"),
    };
    let admitted = serving.admit_physical_work(receipt).unwrap();
    expect_ready(serving.request_physical_work(admitted).unwrap())
}

pub(crate) fn ready_write(
    serving: &ServingPhysicalRuntime,
    request: PhysicalMutationWorkRequest,
) -> ReadyPhysicalWork {
    let receipt = match serving
        .physical_mutation_submission()
        .submit(request)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        outcome => panic!("Phase 16 write submission should succeed: {outcome:?}"),
    };
    let admitted = serving.admit_physical_work(receipt).unwrap();
    expect_ready(serving.request_physical_work(admitted).unwrap())
}

pub(crate) fn admit_read(
    serving: &ServingPhysicalRuntime,
    ready: ReadyPhysicalWork,
) -> ResourceAdmittedPhysicalWork {
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
    let demand = worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::secure_demand(demand, &backend);
    serving
        .admit_physical_scheduler_demand(
            demand,
            &backend,
            worth_store_testkit_store::physical_work::scheduler::policy_evidence::policy_receipt_for(
                requested_budget,
                0,
                worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeRead,
            ),
        )
        .unwrap()
}

pub(crate) fn deny_scheduler_breadth(
    serving: &ServingPhysicalRuntime,
    request: PhysicalReadWorkRequest,
) -> worth_store::physical_runtime::PhysicalWorkIdentity {
    let ready = ready_read(serving, request);
    let identity = ready.intent().identity();
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
    assert!(matches!(
        serving.admit_physical_scheduler_demand(
            demand,
            &backend,
            worth_store_testkit_store::physical_work::scheduler::policy_evidence::exhausted_policy_receipt(
                requested_budget,
                worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeRead,
            ),
        ),
        Err(worth_store::physical_runtime::PhysicalSchedulerDenial::Queue(
            worth_store_io_scheduler::QueueExecutionAdmissionDenial::PolicyReceiptBudgetMismatch { .. }
        ))
    ));
    identity
}

pub(crate) fn retry_write_after_clock_wake(
    serving: &ServingPhysicalRuntime,
    ready: ReadyPhysicalWork,
    payload: &[u8],
) -> worth_store::physical_runtime::PhysicalWorkIdentity {
    let admitted = admit_write(serving, ready);
    let consumer = admitted.consumer_handle();
    let identity = admitted.intent().identity();
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(admitted, payload).unwrap();
    let outcome = serving.execute_physical_work(command).unwrap();
    assert_eq!(
        outcome.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::ProvenNoEffect
    );
    serving
        .advance_physical_signal_clock(
            consumer,
            worth_signal::facade::ClockAdvanceRequest::new(
                worth_signal::facade::ClockDomain::MonotonicExecution,
                worth_signal::facade::ClockTick::new(1_000),
            ),
        )
        .unwrap();
    serving.timeout_physical_work(consumer).unwrap();
    let settled = outcome.into_settled();
    let retry = match serving.schedule_physical_work_retry(&settled).unwrap() {
        worth_store::physical_runtime::PhysicalWorkRetryScheduleOutcome::Scheduled(retry) => retry,
        worth_store::physical_runtime::PhysicalWorkRetryScheduleOutcome::Denied(report) => {
            panic!("Phase 16 pre-effect retry should schedule: {report:?}")
        }
    };
    serving
        .advance_physical_signal_clock(
            consumer,
            worth_signal::facade::ClockAdvanceRequest::new(
                worth_signal::facade::ClockDomain::MonotonicExecution,
                worth_signal::facade::ClockTick::new(1_001),
            ),
        )
        .unwrap();
    let retry = serving.admit_physical_work_retry(&retry, settled).unwrap();
    let (ready, command, _) = retry.into_parts();
    let admitted = admit_write(serving, ready);
    let outcome = serving
        .execute_physical_work(command.bind(admitted).unwrap())
        .unwrap();
    assert_eq!(
        outcome.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
    );
    identity
}

pub(crate) fn execute_exact_writeback(
    serving: &ServingPhysicalRuntime,
    _request: PhysicalMutationWorkRequest,
    bytes: Vec<u8>,
) -> worth_store::physical_runtime::PhysicalWorkIdentity {
    let coordinate =
        worth_store_physical_format::RecordFrameCoordinate::new(worth_store_physical_format::RecordArtifactFile::BootstrapCatalog, 8, 8).unwrap();
    let residency = serving.certification_physical_residency();
    let lease = residency.pin_exact(coordinate).unwrap();
    let dirty = residency
        .admit_dirty_frame(lease, move |_, target| {
            target.copy_from_slice(&bytes);
        })
        .unwrap();
    let prepared = residency
        .prepare_writeback(
            dirty,
            worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        )
        .unwrap();
    let identity = prepared.identity();
    let ready = residency.request_writeback(prepared).unwrap();
    let admitted = residency.admit_writeback(ready).unwrap();
    let outcome = match residency.execute_writeback(admitted).unwrap() {
        worth_store::physical_runtime::PhysicalWritebackExecution::Clean(settlement) => settlement,
        worth_store::physical_runtime::PhysicalWritebackExecution::Retryable(_) => {
            panic!("unfaulted Phase 16 writeback unexpectedly required retry")
        }
        worth_store::physical_runtime::PhysicalWritebackExecution::InspectionRequired(_) => {
            panic!("unfaulted Phase 16 writeback unexpectedly required inspection")
        }
    };
    assert_eq!(
        outcome.effect_fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
    );
    assert_eq!(
        outcome.recovery(),
        worth_store::physical_runtime::PhysicalWorkRecoveryDisposition::ContinueSettlement
    );
    assert_eq!(outcome.signal(), worth_store::physical_runtime::PhysicalSignalSettlementOutcome::Committed);
    assert_eq!(residency.counters().dirty_frames(), 0);
    identity
}

pub(crate) fn admit_write(
    serving: &ServingPhysicalRuntime,
    ready: ReadyPhysicalWork,
) -> ResourceAdmittedPhysicalWork {
    let demand = worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::write_demand(serving, ready);
    let work = demand.queue_work();
    let backend_requirement = work.backend_requirement();
    let requested_budget = work.requested_budget();
    let backend = serving
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let demand = worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::secure_demand(demand, &backend);
    serving
        .admit_physical_scheduler_demand(
            demand,
            &backend,
            worth_store_testkit_store::physical_work::scheduler::policy_evidence::policy_receipt(requested_budget),
        )
        .unwrap()
}

pub(crate) fn expect_ready(readiness: PhysicalWorkReadiness) -> ReadyPhysicalWork {
    match readiness {
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => ready,
        worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(blocked) => {
            panic!(
                "Phase 16 work unexpectedly blocked: {:?}",
                blocked.condition()
            )
        }
    }
}