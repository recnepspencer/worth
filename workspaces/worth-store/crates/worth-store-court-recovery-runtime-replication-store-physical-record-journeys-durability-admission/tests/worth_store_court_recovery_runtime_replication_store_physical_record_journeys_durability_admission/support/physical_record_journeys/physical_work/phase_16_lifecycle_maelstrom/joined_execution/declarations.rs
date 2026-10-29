pub(crate) use {std::collections::HashSet, std::sync::mpsc, std::time::Duration, std::time::Instant};

pub(crate) use {worth_store::physical_runtime::PhysicalEffectObligation, worth_store::physical_runtime::PhysicalExecutorCommand, worth_store::physical_runtime::PhysicalSignalSettlementOutcome, worth_store::physical_runtime::PhysicalWorkConcurrencyRelation, worth_store::physical_runtime::PhysicalWorkConsumerHandle, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::PhysicalWorkIdentity, worth_store::physical_runtime::PhysicalWorkPreEffectDenial, worth_store::physical_runtime::ReadyPhysicalWork, worth_store::physical_runtime::ResourceAdmittedPhysicalWork, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_physical_backend::MediaOperationRole, worth_store_physical_backend::MediaPauseGate};

pub(crate) use {crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::fixture::MaelstromPauseGates, crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::world::MaelstromWorld, crate::physical_work::phase_16_lifecycle_maelstrom::LifecycleMaelstromModel};

pub(crate) struct JoinedTrace {
    pub(crate) denial: PhysicalWorkIdentity,
    pub(crate) pre_dispatch_cancellation: PhysicalWorkIdentity,
    pub(crate) post_dispatch_cancellation: PhysicalWorkIdentity,
}

pub(crate) struct ReorderedReadCommands {
    pub(crate) first: PhysicalExecutorCommand,
    pub(crate) second: PhysicalExecutorCommand,
    pub(crate) first_consumer: PhysicalWorkConsumerHandle,
    pub(crate) first_identity: PhysicalWorkIdentity,
}

pub(crate) fn execute(world: &MaelstromWorld, model: &LifecycleMaelstromModel) -> JoinedTrace {
    let (first, second, write) = apply_one_slice_delta(world);
    let denial =
        crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::deny_scheduler_breadth(&world.serving, world.fixture.reads[0].clone());
    let pre_dispatch_cancellation = prove_pre_dispatch_cancellation(
        &world.serving,
        crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::ready_read(&world.serving, world.fixture.reads[1].clone()),
    );
    let post_dispatch_cancellation = prove_reordered_reads(world, first, second, model);
    let retry = prove_clock_retry(world, write, model);
    let writeback = prove_exact_writeback(world, model);
    assert_causal_evidence(world, retry, writeback);
    JoinedTrace {
        denial,
        pre_dispatch_cancellation,
        post_dispatch_cancellation,
    }
}

pub(crate) fn apply_one_slice_delta(
    world: &MaelstromWorld,
) -> (ReadyPhysicalWork, ReadyPhysicalWork, ReadyPhysicalWork) {
    let first = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::ready_read(&world.serving, world.fixture.reads[0].clone());
    let second = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::ready_read(&world.serving, world.fixture.reads[1].clone());
    let write = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::ready_write(&world.serving, world.fixture.write.clone());
    let lineages = [
        first.signal_request(),
        second.signal_request(),
        write.signal_request(),
    ];
    let before = world.serving.media_counters();
    world
        .serving
        .certification_apply_physical_aspect_delta(world.fixture.read_delta.clone())
        .unwrap();
    let first = revalidate(&world.serving, first);
    let second = revalidate(&world.serving, second);
    let write = revalidate(&world.serving, write);
    assert_ne!(first.signal_request(), lineages[0]);
    assert_eq!(second.signal_request(), lineages[1]);
    assert_eq!(write.signal_request(), lineages[2]);
    assert_eq!(world.serving.media_counters(), before);
    (first, second, write)
}

pub(crate) fn revalidate(serving: &ServingPhysicalRuntime, ready: ReadyPhysicalWork) -> ReadyPhysicalWork {
    crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::expect_ready(serving.revalidate_physical_work(ready).unwrap())
}

pub(crate) fn prove_pre_dispatch_cancellation(
    serving: &ServingPhysicalRuntime,
    ready: ReadyPhysicalWork,
) -> PhysicalWorkIdentity {
    let admitted = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::admit_read(serving, ready);
    let consumer = admitted.consumer_handle();
    let identity = admitted.intent().identity();
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::read(admitted).unwrap();
    let before = serving.media_counters();
    let cancelled = serving.cancel_physical_work(consumer).unwrap();
    assert_eq!(
        cancelled.obligation(),
        worth_store::physical_runtime::PhysicalEffectObligation::NotDispatched
    );
    assert!(matches!(
        serving.execute_physical_work(command),
        Err(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::ConsumerCancelled)
    ));
    assert_eq!(serving.media_counters(), before);
    identity
}

pub(crate) fn prove_reordered_reads(
    world: &MaelstromWorld,
    first: ReadyPhysicalWork,
    second: ReadyPhysicalWork,
    model: &LifecycleMaelstromModel,
) -> PhysicalWorkIdentity {
    let before = world.serving.media_counters();
    let (first, second, cancelled) =
        execute_reordered_reads(&world.serving, first, second, &world.gates);
    assert_eq!(
        [
            first.settled().evidence().fate(),
            second.settled().evidence().fate(),
        ],
        [
            worth_store::physical_runtime::PhysicalWorkEffectFate::ReadCompleted,
            worth_store::physical_runtime::PhysicalWorkEffectFate::ReadCompleted,
        ]
    );
    let after = world.serving.media_counters();
    assert_eq!(
        after.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead),
        model.disjoint_read_effects
    );
    cancelled
}

pub(crate) fn prove_clock_retry(
    world: &MaelstromWorld,
    write: ReadyPhysicalWork,
    model: &LifecycleMaelstromModel,
) -> PhysicalWorkIdentity {
    let before = world.serving.media_counters();
    let identity = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::retry_write_after_clock_wake(
        &world.serving,
        write,
        &world.original_writeback,
    );
    let after = world.serving.media_counters();
    assert_eq!(
        after.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        model.retry_write_attempts
    );
    assert_eq!(
        after.completed_operations_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.completed_operations_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        model.retry_write_completions
    );
    identity
}

pub(crate) fn prove_exact_writeback(
    world: &MaelstromWorld,
    model: &LifecycleMaelstromModel,
) -> PhysicalWorkIdentity {
    let before = world.serving.media_counters();
    let identity = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::execute_exact_writeback(
        &world.serving,
        world.fixture.write.clone(),
        world.original_writeback.clone(),
    );
    let after = world.serving.media_counters();
    assert_eq!(
        after.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        model.exact_writeback_effects
    );
    assert_eq!(
        &std::fs::read(&world.catalog_path).unwrap()[8..16],
        world.original_writeback
    );
    identity
}

pub(crate) fn assert_causal_evidence(
    world: &MaelstromWorld,
    retry: PhysicalWorkIdentity,
    writeback: PhysicalWorkIdentity,
) {
    let causal = world.observer.causal().records();
    assert!(causal.iter().any(|record| record.identity() == retry));
    assert!(causal.iter().any(|record| record.identity() == writeback));
    assert!(causal.iter().all(|record| {
        record.derived_completion()
            != Some(worth_store::physical_runtime::PhysicalSignalSettlementOutcome::DerivedStateUnavailable)
    }));
    let backend = causal
        .iter()
        .filter_map(|record| record.backend_operation())
        .collect::<Vec<_>>();
    assert_eq!(
        backend.iter().copied().collect::<HashSet<_>>().len(),
        backend.len(),
        "one backend effect cannot settle two physical attempts"
    );
}

pub(crate) fn execute_reordered_reads(
    serving: &ServingPhysicalRuntime,
    first: ReadyPhysicalWork,
    second: ReadyPhysicalWork,
    gates: &MaelstromPauseGates,
) -> (
    worth_store::physical_runtime::PhysicalWorkExecutionOutcome,
    worth_store::physical_runtime::PhysicalWorkExecutionOutcome,
    PhysicalWorkIdentity,
) {
    let commands = prepare_reordered_reads(serving, first, second);
    let first_execution = serving.physical_work_execution();
    let second_execution = serving.physical_work_execution();
    let (completed, observed) = std::sync::mpsc::sync_channel(2);
    std::thread::scope(|scope| {
        let first_completed = completed.clone();
        scope.spawn(move || {
            first_completed
                .send((1, first_execution.execute_physical_work(commands.first)))
                .unwrap();
        });
        require_gate(&gates.first_read, &gates.second_read, "first disjoint read");
        scope.spawn(move || {
            completed
                .send((2, second_execution.execute_physical_work(commands.second)))
                .unwrap();
        });
        require_gate(
            &gates.second_read,
            &gates.first_read,
            "second disjoint read",
        );
        let cancellation = serving
            .cancel_physical_work(commands.first_consumer)
            .unwrap();
        assert_eq!(
            cancellation.obligation(),
            worth_store::physical_runtime::PhysicalEffectObligation::SettlementContinues
        );
        gates.second_read.release();
        let second = observed.recv_timeout(std::time::Duration::from_secs(3)).unwrap();
        if second.0 != 2 {
            gates.first_read.release();
            panic!("the released disjoint read must complete first");
        }
        gates.first_read.release();
        let first = observed.recv_timeout(std::time::Duration::from_secs(3)).unwrap();
        assert_eq!(first.0, 1);
        (first.1.unwrap(), second.1.unwrap(), commands.first_identity)
    })
}

pub(crate) fn prepare_reordered_reads(
    serving: &ServingPhysicalRuntime,
    first: ReadyPhysicalWork,
    second: ReadyPhysicalWork,
) -> ReorderedReadCommands {
    let first = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::admit_read(serving, first);
    let second = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::admit_read(serving, second);
    assert_disjoint(&first, &second);
    ReorderedReadCommands {
        first_consumer: first.consumer_handle(),
        first_identity: first.intent().identity(),
        first: worth_store::physical_runtime::PhysicalExecutorCommand::read(first).unwrap(),
        second: worth_store::physical_runtime::PhysicalExecutorCommand::read(second).unwrap(),
    }
}

pub(crate) fn assert_disjoint(first: &ResourceAdmittedPhysicalWork, second: &ResourceAdmittedPhysicalWork) {
    assert_eq!(
        first
            .concurrency_scope()
            .relation(&second.concurrency_scope()),
        worth_store::physical_runtime::PhysicalWorkConcurrencyRelation::DisjointArtifacts
    );
}

pub(crate) fn require_gate(gate: &MediaPauseGate, peer: &MediaPauseGate, label: &str) {
    if !gate_reached_within(gate) {
        gate.release();
        peer.release();
        panic!("{label} did not reach its independent backend gate");
    }
}

pub(crate) fn gate_reached_within(gate: &MediaPauseGate) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while gate.reached_context().is_none() && std::time::Instant::now() < deadline {
        std::thread::yield_now();
    }
    gate.reached_context().is_some()
}