#[allow(unused_imports)]
use crate::phase_four_planning::phase_six_publication_counters::*;

#[test]
fn second_candidate_materialization_failure_retains_the_first_completed_candidate() {
    let outcome = publication_fault_outcome(
        "c8-phase6-candidate-materialization-prefix",
        worth_store::physical_runtime::certification::MediaOperationRole::PositionedWrite,
        2,
        worth_store::physical_runtime::certification::MediaFaultDirective::AllowPrefix {
            bytes: 31,
        },
    );
    let worth_store_recovery_runtime::PhysicalRecoveryPublicationSettlement::Indeterminate(
        worth_store::physical_runtime::PhysicalRecoveryPublicationCommandIndeterminate::CandidateMaterialization {
            completed,
            ..
        },
    ) = outcome.settlement().settlement()
    else {
        panic!("second candidate write must retain a materialization prefix")
    };
    assert_eq!(completed.len(), 1);
    assert_prefix(&outcome, 1, 1, 1, 0);
}

#[test]
fn candidate_sync_indeterminate_retains_its_completed_materialization() {
    let outcome = publication_fault_outcome(
        "c8-phase6-candidate-sync-indeterminate",
        worth_store::physical_runtime::certification::MediaOperationRole::SynchronizeFileState,
        1,
        worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
    );
    assert!(matches!(
        outcome.settlement().settlement(),
        worth_store_recovery_runtime::PhysicalRecoveryPublicationSettlement::Indeterminate(
            worth_store::physical_runtime::PhysicalRecoveryPublicationCommandIndeterminate::CandidateSynchronization { .. }
        )
    ));
    assert_prefix(&outcome, 0, 1, 0, 0);
}

#[test]
fn candidate_sync_denial_retains_the_escaped_materialization_prefix() {
    let outcome = publication_fault_outcome(
        "c8-phase6-candidate-sync-denial",
        worth_store::physical_runtime::certification::MediaOperationRole::SynchronizeFileState,
        1,
        worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    assert!(matches!(
        outcome.settlement().settlement(),
        worth_store_recovery_runtime::PhysicalRecoveryPublicationSettlement::DeniedBeforeEffect(denial)
            if denial.candidate_materialization().is_some()
    ));
    assert_prefix(&outcome, 0, 1, 0, 0);
}

#[test]
fn candidate_sync_scheduler_failure_retains_the_completed_materialization() {
    let retained_root = crate::phase_four_planning::prepare_ordinary_recovery_root(
        "c8-phase6-candidate-sync-signal",
    );
    let staged = crate::phase_four_planning::selected_ordinary_recovery(retained_root.path())
        .plan()
        .unwrap()
        .stage()
        .unwrap();
    staged.certification_fail_publication_scheduler_settlement_at(
        worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage::CandidateSynchronization,
    );
    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
        outcome,
    )) = staged.publish()
    else {
        panic!("candidate synchronization scheduler failure must be indeterminate")
    };
    assert!(matches!(
        outcome.settlement().settlement(),
        worth_store_recovery_runtime::PhysicalRecoveryPublicationSettlement::Indeterminate(
            worth_store::physical_runtime::PhysicalRecoveryPublicationCommandIndeterminate::CandidateSynchronizationSettlement { .. }
        )
    ));
    assert_prefix(&outcome, 0, 1, 0, 0);
}

#[test]
fn namespace_scheduler_failure_retains_candidates_and_root_protocol() {
    let retained_root =
        crate::phase_four_planning::prepare_ordinary_recovery_root("c8-phase6-namespace-scheduler");
    let staged = crate::phase_four_planning::selected_ordinary_recovery(retained_root.path())
        .plan()
        .unwrap()
        .stage()
        .unwrap();
    let expected_candidates = staged.publication_plan().candidates().len() as u64;
    staged.certification_fail_publication_scheduler_settlement_at(
        worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage::RecordNamespaceSynchronization,
    );
    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
        outcome,
    )) = staged.publish()
    else {
        panic!("namespace scheduler failure must retain the completed publication prefix")
    };
    assert!(matches!(
        outcome.settlement().settlement(),
        worth_store_recovery_runtime::PhysicalRecoveryPublicationSettlement::Indeterminate(
            worth_store::physical_runtime::PhysicalRecoveryPublicationCommandIndeterminate::Scheduler { .. }
        )
    ));
    assert_prefix(
        &outcome,
        expected_candidates,
        expected_candidates,
        expected_candidates,
        1,
    );
    assert_eq!(outcome.counters().namespace_synchronizations_performed, 0);
}

#[test]
fn candidate_materialization_signal_failure_retains_its_physical_settlement() {
    let (outcome, _) = publication_signal_outcome(
        "c8-phase6-candidate-materialization-signal",
        worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage::CandidateMaterialization,
    );
    assert!(matches!(
        outcome.settlement().settlement(),
        worth_store_recovery_runtime::PhysicalRecoveryPublicationSettlement::Indeterminate(
            worth_store::physical_runtime::PhysicalRecoveryPublicationCommandIndeterminate::CandidateMaterializationSettlement {
                failure: worth_store::physical_runtime::PhysicalRecoveryPublicationSettlementFailure::Signal(
                    worth_store::physical_runtime::PhysicalSignalSettlementOutcome::DerivedStateUnavailable
                ),
                ..
            }
        )
    ));
    assert_prefix(&outcome, 0, 0, 0, 0);
}

#[test]
fn candidate_synchronization_signal_failure_retains_materialization() {
    let (outcome, _) = publication_signal_outcome(
        "c8-phase6-candidate-synchronization-signal",
        worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage::CandidateSynchronization,
    );
    assert!(matches!(
        outcome.settlement().settlement(),
        worth_store_recovery_runtime::PhysicalRecoveryPublicationSettlement::Indeterminate(
            worth_store::physical_runtime::PhysicalRecoveryPublicationCommandIndeterminate::CandidateSynchronizationSettlement {
                failure: worth_store::physical_runtime::PhysicalRecoveryPublicationSettlementFailure::Signal(
                    worth_store::physical_runtime::PhysicalSignalSettlementOutcome::DerivedStateUnavailable
                ),
                ..
            }
        )
    ));
    assert_prefix(&outcome, 0, 1, 0, 0);
}

#[test]
fn root_signal_failure_retains_every_completed_candidate() {
    let (outcome, expected_candidates) = publication_signal_outcome(
        "c8-phase6-root-signal",
        worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage::RootProtocolReplacement,
    );
    assert_signal_stage(
        &outcome,
        worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage::RootProtocolReplacement,
    );
    assert_prefix(
        &outcome,
        expected_candidates,
        expected_candidates,
        expected_candidates,
        0,
    );
}

#[test]
fn namespace_signal_failure_retains_candidates_and_root_protocol() {
    let (outcome, expected_candidates) = publication_signal_outcome(
        "c8-phase6-namespace-signal",
        worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage::RecordNamespaceSynchronization,
    );
    assert_signal_stage(
        &outcome,
        worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage::RecordNamespaceSynchronization,
    );
    assert_prefix(
        &outcome,
        expected_candidates,
        expected_candidates,
        expected_candidates,
        1,
    );
    assert_eq!(outcome.counters().namespace_synchronizations_performed, 0);
}
