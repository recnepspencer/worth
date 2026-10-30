pub(crate) use {
    worth_store::physical_runtime::certification::MediaFaultDirective,
    worth_store::physical_runtime::certification::MediaOperationRole,
};
pub(crate) use {
    worth_store::physical_runtime::PhysicalRecoveryPublicationCommandIndeterminate,
    worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage,
    worth_store::physical_runtime::PhysicalRecoveryPublicationSettlementFailure,
    worth_store::physical_runtime::PhysicalSignalSettlementOutcome,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryPublicationIndeterminate,
    worth_store_recovery_runtime::PhysicalRecoveryPublicationSettlement,
};

pub(crate) fn publication_signal_outcome(
    label: &str,
    stage: PhysicalRecoveryPublicationCommandStage,
) -> (PhysicalRecoveryPublicationIndeterminate, u64) {
    let retained_root = crate::phase_four_planning::prepare_ordinary_recovery_root(label);
    let staged = crate::phase_four_planning::selected_ordinary_recovery(retained_root.path())
        .plan()
        .unwrap()
        .stage()
        .unwrap();
    let expected_candidates = staged.publication_plan().candidates().len() as u64;
    staged.certification_fail_publication_signal_settlement_at(stage);
    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
        outcome,
    )) = staged.publish()
    else {
        panic!("publication Signal failure must retain indeterminate evidence")
    };
    (*outcome, expected_candidates)
}

pub(crate) fn assert_signal_stage(
    outcome: &PhysicalRecoveryPublicationIndeterminate,
    expected: PhysicalRecoveryPublicationCommandStage,
) {
    assert!(matches!(
        outcome.settlement().settlement(),
        worth_store_recovery_runtime::PhysicalRecoveryPublicationSettlement::Indeterminate(
            worth_store::physical_runtime::PhysicalRecoveryPublicationCommandIndeterminate::Signal {
                stage,
                outcome: worth_store::physical_runtime::PhysicalSignalSettlementOutcome::DerivedStateUnavailable,
                ..
            }
        ) if *stage == expected
    ));
}

pub(crate) fn publication_fault_outcome(
    label: &str,
    role: MediaOperationRole,
    occurrence: u64,
    directive: MediaFaultDirective,
) -> PhysicalRecoveryPublicationIndeterminate {
    let retained_root = crate::phase_four_planning::prepare_ordinary_recovery_root(label);
    let (selected, activation) =
        crate::phase_four_planning::publication_faults::selected_with_publication_fault(
            retained_root.path(),
            role,
            occurrence,
            directive,
        );
    let staged = selected.plan().unwrap().stage().unwrap();
    activation.arm().unwrap();
    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
        outcome,
    )) = staged.publish()
    else {
        panic!("the activated publication fault must retain indeterminate evidence")
    };
    *outcome
}

pub(crate) fn assert_prefix(
    outcome: &PhysicalRecoveryPublicationIndeterminate,
    candidates: u64,
    materializations: u64,
    synchronizations: u64,
    root_protocol: u64,
) {
    let counters = outcome.counters();
    assert_eq!(counters.candidate_artifacts_settled, candidates);
    assert_eq!(
        counters.candidate_materializations_performed,
        materializations
    );
    assert_eq!(
        counters.candidate_synchronizations_performed,
        synchronizations
    );
    assert_eq!(counters.root_protocol_replacements_performed, root_protocol);
}
