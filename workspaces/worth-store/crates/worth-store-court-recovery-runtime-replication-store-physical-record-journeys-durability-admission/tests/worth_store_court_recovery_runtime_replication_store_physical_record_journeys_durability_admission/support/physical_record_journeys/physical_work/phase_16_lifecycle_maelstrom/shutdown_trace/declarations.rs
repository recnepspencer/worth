pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalSignalPauseGate, worth_store::physical_runtime::CertificationFrameReadFailure, worth_store::physical_runtime::CertificationFrameWorkFailure, worth_store::physical_runtime::PhysicalExecutorCommand, worth_store::physical_runtime::PhysicalResidencyCertification, worth_store::physical_runtime::PhysicalStoreCloseOutcome, worth_store::physical_runtime::PhysicalStoreClosePhase, worth_store::physical_runtime::PhysicalStoreClosePlan, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::PhysicalWorkExecution, worth_store::physical_runtime::PhysicalWorkExecutionOutcome, worth_store::physical_runtime::PhysicalWorkIdentity, worth_store::physical_runtime::PhysicalWorkPreEffectDenial};
pub(crate) use {worth_store_physical_backend::MediaPauseGate};
pub(crate) use {worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordFrameCoordinate};

pub(crate) use {crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::joined_execution::JoinedTrace, crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::world::MaelstromWorld, crate::physical_work::phase_16_lifecycle_maelstrom::LifecycleMaelstromModel};

pub(crate) fn close_and_finish(
    world: MaelstromWorld,
    trace: JoinedTrace,
    model: &LifecycleMaelstromModel,
) {
    let fresh = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::fresh_process::spawn(&world.root);
    let close_read = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::ready_read(&world.serving, world.fixture.reads[0].clone());
    let close_read = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::admit_read(&world.serving, close_read);
    let close_read_identity = close_read.intent().identity();
    let close_read = worth_store::physical_runtime::PhysicalExecutorCommand::read(close_read).unwrap();
    let close_execution = world.serving.physical_work_execution();
    let hot_pin = prewarmed_writeback_pin(&world.serving);
    let abandoned = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::workflows::ready_read(&world.serving, world.fixture.reads[0].clone());
    let abandoned_identity = abandoned.intent().identity();
    let signal_gate = world
        .serving
        .certification_pause_physical_signal_after_dequeue();
    drop(abandoned);
    assert!(signal_gate.await_arrivals(1));
    let close = world.serving.close_plan();
    world.gates.close_read_activation.arm().unwrap();
    let (closed, close_read) = execute_dispatched_close(
        close,
        close_execution,
        close_read,
        &world.gates.close_read,
        signal_gate,
        hot_pin,
    );
    assert!(world.gates.close_read_activation.is_consumed());
    assert_eq!(
        close_read.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::ReadCompleted
    );
    assert!(matches!(closed, worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }));
    assert_shutdown(&closed, &trace, abandoned_identity, close_read_identity);
    let fresh = fresh.open_after_close();
    assert_ne!(fresh.process.get(), std::process::id());
    assert_eq!(fresh.root_generation, model.fresh_root_generation);
    let mut actual_records = fresh.records;
    actual_records.sort();
    let mut expected_records = model
        .fresh_records
        .iter()
        .map(|record| record.to_vec())
        .collect::<Vec<_>>();
    expected_records.sort();
    assert_eq!(actual_records, expected_records);
}

pub(crate) struct HotPinFence {
    pub(crate) residency: PhysicalResidencyCertification,
    pub(crate) coordinate: RecordFrameCoordinate,
}

pub(crate) fn prewarmed_writeback_pin(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) -> HotPinFence {
    let residency = serving.certification_physical_residency();
    let coordinate =
        worth_store_physical_format::RecordFrameCoordinate::new(worth_store_physical_format::RecordArtifactFile::BootstrapCatalog, 8, 8).unwrap();
    let hot = residency.pin_exact(coordinate).unwrap();
    assert_eq!(hot.physical_work_count(), 0);
    drop(hot);
    HotPinFence {
        residency,
        coordinate,
    }
}

pub(crate) fn execute_dispatched_close(
    close: PhysicalStoreClosePlan,
    execution: PhysicalWorkExecution,
    command: PhysicalExecutorCommand,
    media_gate: &MediaPauseGate,
    signal_gate: CertificationPhysicalSignalPauseGate,
    hot_pin: HotPinFence,
) -> (PhysicalStoreCloseOutcome, PhysicalWorkExecutionOutcome) {
    let progress = close.observation();
    std::thread::scope(|scope| {
        let effect = scope.spawn(move || execution.execute_physical_work(command));
        wait_until(|| media_gate.reached_context().is_some());
        let gated_operation = media_gate
            .reached_context()
            .and_then(|context| context.operation())
            .expect("dispatched close gate must bind an identified operation");
        let closing = scope.spawn(move || close.execute());
        wait_until(|| progress.reached(worth_store::physical_runtime::PhysicalStoreClosePhase::AdmissionStopped));
        assert!(matches!(
            hot_pin.residency.pin_exact(hot_pin.coordinate),
            Err(worth_store::physical_runtime::CertificationFrameReadFailure::PhysicalWork(
                worth_store::physical_runtime::CertificationFrameWorkFailure::PreEffect(
                    worth_store::physical_runtime::PhysicalWorkPreEffectDenial::AdmissionStopped
                )
            ))
        ));
        assert!(!progress.reached(worth_store::physical_runtime::PhysicalStoreClosePhase::DispatchSettlementComplete));
        assert!(!progress.reached(worth_store::physical_runtime::PhysicalStoreClosePhase::SignalDisposed));
        assert!(!progress.reached(worth_store::physical_runtime::PhysicalStoreClosePhase::MediaReleased));
        media_gate.release();
        signal_gate.release();
        let closed = closing.join().unwrap();
        let effect = effect.join().unwrap().unwrap();
        assert_eq!(
            effect
                .settled()
                .effect_identity()
                .expect("dispatched close read must retain its backend identity")
                .backend_operation(),
            gated_operation
        );
        (closed, effect)
    })
}

pub(crate) fn assert_shutdown(
    closed: &PhysicalStoreCloseOutcome,
    trace: &JoinedTrace,
    abandoned: worth_store::physical_runtime::PhysicalWorkIdentity,
    dispatched_close: PhysicalWorkIdentity,
) {
    let drain = closed.shutdown().work().drain();
    assert!(drain
        .continued_after_consumer_cancellation()
        .contains(&trace.post_dispatch_cancellation));
    assert!(drain
        .cancelled_before_dispatch()
        .contains(&trace.pre_dispatch_cancellation));
    assert!(drain.released_before_dispatch().contains(&trace.denial));
    assert!(drain.released_before_dispatch().contains(&abandoned));
    assert!(drain.settled().contains(&dispatched_close));
    assert_eq!(closed.shutdown().work().residual(), 0);
    assert_eq!(
        closed
            .shutdown()
            .signal_summary()
            .unwrap()
            .active_in_flight_node_count(),
        0
    );
}

pub(crate) fn wait_until(mut predicate: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !predicate() && std::time::Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(predicate(), "Phase 16 maelstrom missed a bounded event");
}