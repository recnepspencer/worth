use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed,
    worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare,
    worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare_with_deadline,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
};

#[test]
fn cancellation_after_each_possible_effect_boundary_remains_effectful() {
    for (index, checkpoint) in [
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterGroupSeal,
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::DuringDataSettlement,
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::DuringRootPublication,
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeTerminalFinalization,
    ]
    .into_iter()
    .enumerate()
    {
        let parent = tempfile::tempdir().unwrap();
        let serving = serving_from_initialization(&parent.path().join("store"));
        let (_, placement, _) = configuration();
        let gate = serving.certification_pause_physical_mutation_at(checkpoint);
        let handle = prepare(
            &serving,
            placement,
            [200 + index as u8; 32],
            b"effectful-cancellation-matrix",
        )
        .start();
        assert!(
            gate.await_arrival(),
            "mutation did not reach {checkpoint:?}"
        );
        assert!(matches!(
            handle.request_cancellation(),
            worth_store::physical_runtime::PhysicalMutationCancellationOutcome::SettlementAlreadyEffectful { .. }
        ));
        gate.release();
        completed(handle.wait());
        let shutdown = serving.close().mutations();
        assert_eq!(shutdown.completed(), 1);
        assert_eq!(shutdown.cancellation_effectful(), 1);
    }
}

#[test]
fn deadline_elapsed_after_group_seal_cannot_rewrite_effectful_fate() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let gate = serving.certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterGroupSeal,
    );
    let handle =
        prepare_with_deadline(&serving, placement, [210; 32], b"post-effect-deadline", 7).start();
    assert!(gate.await_arrival());
    serving
        .certification_advance_physical_signal_clock(
            worth_signal::facade::ClockAdvanceRequest::new(
                worth_signal::facade::ClockDomain::MonotonicExecution,
                worth_signal::facade::ClockTick::new(7),
            ),
        )
        .unwrap();
    gate.release();
    completed(handle.wait());
    let shutdown = serving.close().mutations();
    assert_eq!(shutdown.completed(), 1);
    assert_eq!(shutdown.proven_no_effect(), 0);
}
