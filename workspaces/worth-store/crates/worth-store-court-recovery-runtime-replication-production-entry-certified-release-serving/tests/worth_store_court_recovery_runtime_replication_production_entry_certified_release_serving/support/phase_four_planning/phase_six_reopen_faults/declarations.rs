pub(crate) use worth_store_recovery_runtime::PhysicalRecoveryOutcome;
pub(crate) use {
    worth_store::physical_runtime::PhysicalRecoveryFreshReopenDenialKind,
    worth_store::physical_runtime::PhysicalRecoveryFreshReopenStage,
    worth_store::physical_runtime::PhysicalSignalSettlementOutcome,
    worth_store::physical_runtime::PhysicalWorkSchedulerPosture,
};

pub(crate) fn assert_reopen_scheduler_failure(
    stage: PhysicalRecoveryFreshReopenStage,
    selector_reads: u64,
    root_reads: u64,
) {
    let retained_root = crate::phase_four_planning::prepare_ordinary_recovery_root(&format!(
        "c8-phase6-reopen-scheduler-{stage:?}"
    ));
    let published = crate::phase_four_planning::selected_ordinary_recovery(retained_root.path())
        .plan()
        .unwrap()
        .stage()
        .unwrap()
        .publish()
        .unwrap();
    published.certification_fail_reopen_scheduler_settlement_at(stage);

    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
        outcome,
    )) = published.reopen()
    else {
        panic!("a nonterminal reopen Signal settlement must preserve indeterminate evidence")
    };
    let failure = outcome
        .reopen_failure()
        .expect("fresh-reopen failure evidence");
    assert_eq!(failure.counters().selector_reads_completed, selector_reads);
    assert_eq!(failure.counters().root_reads_completed, root_reads);
    assert!(failure.counters().bytes_read > 0);
    assert_eq!(
        failure.denial().kind(),
        worth_store::physical_runtime::PhysicalRecoveryFreshReopenDenialKind::SchedulerSettlement(
            worth_store::physical_runtime::PhysicalWorkSchedulerPosture::RejectedAfterEffect,
        )
    );
    assert!(outcome.recovery_effects() > 0);
}

pub(crate) fn assert_reopen_signal_failure(
    stage: PhysicalRecoveryFreshReopenStage,
    selector_reads: u64,
    root_reads: u64,
) {
    let retained_root = crate::phase_four_planning::prepare_ordinary_recovery_root(&format!(
        "c8-phase6-reopen-signal-{stage:?}"
    ));
    let published = crate::phase_four_planning::selected_ordinary_recovery(retained_root.path())
        .plan()
        .unwrap()
        .stage()
        .unwrap()
        .publish()
        .unwrap();
    published.certification_fail_reopen_signal_settlement_at(stage);

    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
        outcome,
    )) = published.reopen()
    else {
        panic!("a nonterminal reopen Signal settlement must preserve indeterminate evidence")
    };
    let failure = outcome
        .reopen_failure()
        .expect("fresh-reopen failure evidence");
    assert_eq!(failure.counters().selector_reads_completed, selector_reads);
    assert_eq!(failure.counters().root_reads_completed, root_reads);
    assert!(failure.counters().bytes_read > 0);
    assert_eq!(
        failure.denial().kind(),
        worth_store::physical_runtime::PhysicalRecoveryFreshReopenDenialKind::SignalSettlement(
            worth_store::physical_runtime::PhysicalSignalSettlementOutcome::DerivedStateUnavailable,
        )
    );
    assert!(outcome.recovery_effects() > 0);
}
