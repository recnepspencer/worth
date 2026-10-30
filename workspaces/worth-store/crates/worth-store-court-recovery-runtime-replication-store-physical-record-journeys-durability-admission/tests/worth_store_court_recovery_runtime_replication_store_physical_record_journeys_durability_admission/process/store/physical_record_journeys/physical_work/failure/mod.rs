use tempfile::tempdir;
use {
    worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write,
    worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_positioned_write_fault,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::secure_demand,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::write_demand,
    worth_store_testkit_store::physical_work::scheduler::policy_evidence::policy_receipt,
};

#[test]
fn pre_effect_backend_denial_is_the_only_retryable_physical_failure() {
    let root = tempdir().unwrap();
    let (profile, _, mutation_request) = work_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_positioned_write_fault(
        root.path(),
        profile,
        worth_store_physical_backend::MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let admitted = admitted_write(&serving, mutation_request);
    let consumer = admitted.consumer_handle();
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"retry001".as_slice(),
    )
    .unwrap();

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
    let retry = match serving
        .schedule_physical_work_retry(&settled)
        .unwrap_or_else(|denial| {
            panic!(
                "C5_PREDICATE:store-local-async-registry: local retry state overrode Signal scheduling: {denial:?}"
            )
        })
    {
        worth_store::physical_runtime::PhysicalWorkRetryScheduleOutcome::Scheduled(retry) => retry,
        worth_store::physical_runtime::PhysicalWorkRetryScheduleOutcome::Denied(report) => {
            panic!("pre-effect retry should schedule: {report:?}")
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
    let admission = serving.admit_physical_work_retry(&retry, settled).unwrap();
    let retry_consumer = admission
        .consumer_handle()
        .expect("Signal admitted the retry generation");
    assert_ne!(retry_consumer.signal_request(), consumer.signal_request());
    let (ready, retry_command, _signal) = admission.into_parts();
    let demand = write_demand(&serving, ready);
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
    let retry_outcome = serving
        .execute_physical_work(retry_command.bind(admitted).unwrap())
        .unwrap();
    assert_eq!(
        retry_outcome.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
    );
    assert!(matches!(
        serving.close_plan().execute(),
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

#[test]
fn indeterminate_write_obligation_survives_orderly_close_and_reopen() {
    let root = tempdir().unwrap();
    let (profile, _, mutation_request) = work_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let catalog = std::fs::read(root.path().join("families/records/bootstrap.catalog")).unwrap();
    let serving = serving_from_open_with_positioned_write_fault(
        root.path(),
        profile.clone(),
        worth_store_physical_backend::MediaFaultDirective::AllowPrefix { bytes: 3 },
    );
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted_write(&serving, mutation_request),
        catalog[8..16].to_vec(),
    )
    .unwrap();

    assert_eq!(
        serving
            .execute_physical_work(command)
            .unwrap()
            .settled()
            .evidence()
            .fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::Indeterminate
    );
    assert!(serving.close_plan().execute().requires_inspection());

    let reopened = worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile(root.path(), profile);
    assert!(
        reopened.close_plan().execute().requires_inspection(),
        "a restart must not erase the unresolved physical effect"
    );
}

#[test]
fn partial_write_retains_exact_prefix_and_revokes_serving_health() {
    let root = tempdir().unwrap();
    let (profile, read_request, mutation_request) = work_fixture();
    let profile = profile.with_capacity(
        worth_store::physical_runtime::PhysicalWorkCapacity::new(
            1,
            256,
            256,
            1024 * 1024,
            1024 * 1024,
        )
        .unwrap()
        .with_terminal_evidence_capacity(4)
        .unwrap(),
    );
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let catalog = root.path().join("families/records/bootstrap.catalog");
    let before = std::fs::read(&catalog).unwrap();
    let serving = serving_from_open_with_positioned_write_fault(
        root.path(),
        profile,
        worth_store_physical_backend::MediaFaultDirective::AllowPrefix { bytes: 3 },
    );
    let admitted = admitted_write(&serving, mutation_request);
    let identity = admitted.intent().identity();
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"partial!".as_slice(),
    )
    .unwrap();

    let outcome = serving.execute_physical_work(command).unwrap();

    let worth_store::physical_runtime::PhysicalWorkSettlementEvidence::TerminalFailure(failure) =
        outcome.settled().evidence()
    else {
        panic!("partial write must retain terminal failure evidence");
    };
    assert_eq!(failure.identity(), identity);
    assert_eq!(
        failure.effect_fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::Indeterminate
    );
    assert_eq!(failure.completed_bytes(), 3);
    assert_eq!(
        failure.recovery(),
        worth_store::physical_runtime::PhysicalWorkRecoveryDisposition::InspectionRequired
    );
    assert_eq!(
        failure.scheduler(),
        worth_store::physical_runtime::PhysicalWorkSchedulerPosture::NotObserved
    );
    let observed = std::fs::read(&catalog).unwrap();
    assert_eq!(&observed[8..11], b"par");
    assert_eq!(&observed[11..16], &before[11..16]);

    let receipt = match serving
        .physical_read_submission()
        .submit(read_request.clone())
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        other => panic!("submission remains separately observable: {other:?}"),
    };
    let rejected_identity = receipt.identity();
    assert!(
        matches!(
            serving.admit_physical_work(receipt),
            Err(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::UnhealthyServing)
        ),
        "C5_PREDICATE:health-revocation: indeterminate physical truth must revoke later admission"
    );
    let retry_receipt = match serving
        .physical_read_submission()
        .submit(read_request)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        other => panic!("denied admission must return its one-command capacity: {other:?}"),
    };
    let retry_identity = retry_receipt.identity();
    assert!(matches!(
        serving.admit_physical_work(retry_receipt),
        Err(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::UnhealthyServing)
    ));
    let closed = serving.close_plan().execute();
    assert!(matches!(
        closed,
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::InspectionRequired { .. }
    ));
    assert_eq!(
        closed.shutdown().work().drain().inspection_required(),
        &[identity]
    );
    assert_eq!(
        closed.shutdown().work().drain().cancelled_before_dispatch(),
        &[rejected_identity, retry_identity]
    );
    assert!(closed.shutdown().work().drain().residual().is_empty());
}

#[test]
fn panic_after_dispatch_cannot_be_reclassified_as_a_safe_pre_effect_release() {
    let root = tempdir().unwrap();
    let (profile, read_request, mutation_request) = work_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_positioned_write_fault(
        root.path(),
        profile,
        worth_store_physical_backend::MediaFaultDirective::PanicAfter,
    );
    let admitted = admitted_write(&serving, mutation_request);
    let identity = admitted.intent().identity();
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"unwound!".as_slice(),
    )
    .unwrap();

    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = serving.execute_physical_work(command);
    }));

    assert!(panic.is_err());
    assert_eq!(
        &std::fs::read(root.path().join("families/records/bootstrap.catalog")).unwrap()[8..16],
        b"unwound!"
    );
    let receipt = match serving
        .physical_read_submission()
        .submit(read_request)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        other => panic!("submission remains separately observable: {other:?}"),
    };
    assert!(matches!(
        serving.admit_physical_work(receipt),
        Err(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::UnhealthyServing)
    ));
    let closed = serving.close_plan().execute();
    assert!(matches!(
        closed,
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::InspectionRequired { .. }
    ));
    assert_eq!(
        closed.shutdown().work().drain().inspection_required(),
        &[identity]
    );
    assert!(closed
        .shutdown()
        .work()
        .drain()
        .released_before_dispatch()
        .is_empty());
}

#[test]
fn health_revocation_fences_already_open_and_new_record_reads() {
    let root = tempdir().unwrap();
    let (profile, _, mutation_request) = work_fixture();
    let (_, placement, _) = worth_store_testkit_store::physical_record_journeys::configuration();
    let initial = serving_from_initialization_with_work_profile(root.path(), profile.clone());
    let published = worth_store_testkit_store::durable_publication::root::publish_single(
        &initial,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "health-revocation-read-fence",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"stable".as_slice()])
            .unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    initial.close();

    let serving = serving_from_open_with_positioned_write_fault(
        root.path(),
        profile,
        worth_store_physical_backend::MediaFaultDirective::AllowPrefix { bytes: 3 },
    );
    let mut open = serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(64).unwrap(),
            ),
        )
        .unwrap();
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted_write(&serving, mutation_request),
        b"partial!".as_slice(),
    )
    .unwrap();
    let _ = serving.execute_physical_work(command).unwrap();

    assert!(matches!(
        open.read_next(&mut [0_u8; 6]),
        Err(failure)
            if failure.kind()
                == worth_store::physical_runtime::RecordStreamFailureKind::
                    ServingRequiresInspection
    ));
    assert!(matches!(
        serving.records().expect("read protection admission").open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(64).unwrap(),
            ),
        ),
        Err(error)
            if error.denial()
                == worth_store::physical_runtime::RecordReadDenial::ServingRequiresInspection
    ));
    drop(open);
    assert!(serving.close_plan().execute().requires_inspection());
}
#[path = "derived_reconciliation/mod.rs"]
pub(crate) mod derived_reconciliation;
