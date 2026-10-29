#![cfg(feature = "certification-test-authority")]

use worth_store::physical_runtime::certification::{MediaFaultDirective, MediaOperationRole};
use worth_store::physical_runtime::{
    PhysicalRecoveryPublicationCommandIndeterminate, PhysicalRecoveryPublicationCommandStage,
    PhysicalRecoveryPublicationSettlementFailure, PhysicalSignalSettlementOutcome,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryOutcome, PhysicalRecoveryPublicationIndeterminate,
    PhysicalRecoveryPublicationSettlement,
};

#[test]
fn second_candidate_materialization_failure_retains_the_first_completed_candidate() {
    let outcome = publication_fault_outcome(
        "c8-phase6-candidate-materialization-prefix",
        MediaOperationRole::PositionedWrite,
        2,
        MediaFaultDirective::AllowPrefix { bytes: 31 },
    );
    let PhysicalRecoveryPublicationSettlement::Indeterminate(
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
        MediaOperationRole::SynchronizeFileState,
        1,
        MediaFaultDirective::IndeterminateAfterEffect,
    );
    assert!(matches!(
        outcome.settlement().settlement(),
        PhysicalRecoveryPublicationSettlement::Indeterminate(
            worth_store::physical_runtime::PhysicalRecoveryPublicationCommandIndeterminate::CandidateSynchronization { .. }
        )
    ));
    assert_prefix(&outcome, 0, 1, 0, 0);
}

#[test]
fn candidate_sync_denial_retains_the_escaped_materialization_prefix() {
    let outcome = publication_fault_outcome(
        "c8-phase6-candidate-sync-denial",
        MediaOperationRole::SynchronizeFileState,
        1,
        MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    assert!(matches!(
        outcome.settlement().settlement(),
        PhysicalRecoveryPublicationSettlement::DeniedBeforeEffect(denial)
            if denial.candidate_materialization().is_some()
    ));
    assert_prefix(&outcome, 0, 1, 0, 0);
}

#[test]
fn candidate_sync_scheduler_failure_retains_the_completed_materialization() {
    let retained_root = super::prepare_ordinary_recovery_root("c8-phase6-candidate-sync-signal");
    let staged = super::selected_ordinary_recovery(retained_root.path())
        .plan()
        .unwrap()
        .stage()
        .unwrap();
    staged.certification_fail_publication_scheduler_settlement_at(
        worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage::CandidateSynchronization,
    );
    let Err(PhysicalRecoveryOutcome::PublicationIndeterminate(outcome)) = staged.publish() else {
        panic!("candidate synchronization scheduler failure must be indeterminate")
    };
    assert!(matches!(
        outcome.settlement().settlement(),
        PhysicalRecoveryPublicationSettlement::Indeterminate(
            worth_store::physical_runtime::PhysicalRecoveryPublicationCommandIndeterminate::CandidateSynchronizationSettlement { .. }
        )
    ));
    assert_prefix(&outcome, 0, 1, 0, 0);
}

#[test]
fn namespace_scheduler_failure_retains_candidates_and_root_protocol() {
    let retained_root = super::prepare_ordinary_recovery_root("c8-phase6-namespace-scheduler");
    let staged = super::selected_ordinary_recovery(retained_root.path())
        .plan()
        .unwrap()
        .stage()
        .unwrap();
    let expected_candidates = staged.publication_plan().candidates().len() as u64;
    staged.certification_fail_publication_scheduler_settlement_at(
        worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage::RecordNamespaceSynchronization,
    );
    let Err(PhysicalRecoveryOutcome::PublicationIndeterminate(outcome)) = staged.publish() else {
        panic!("namespace scheduler failure must retain the completed publication prefix")
    };
    assert!(matches!(
        outcome.settlement().settlement(),
        PhysicalRecoveryPublicationSettlement::Indeterminate(
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
        PhysicalRecoveryPublicationCommandStage::CandidateMaterialization,
    );
    assert!(matches!(
        outcome.settlement().settlement(),
        PhysicalRecoveryPublicationSettlement::Indeterminate(
            PhysicalRecoveryPublicationCommandIndeterminate::CandidateMaterializationSettlement {
                failure: PhysicalRecoveryPublicationSettlementFailure::Signal(
                    PhysicalSignalSettlementOutcome::DerivedStateUnavailable
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
        PhysicalRecoveryPublicationCommandStage::CandidateSynchronization,
    );
    assert!(matches!(
        outcome.settlement().settlement(),
        PhysicalRecoveryPublicationSettlement::Indeterminate(
            PhysicalRecoveryPublicationCommandIndeterminate::CandidateSynchronizationSettlement {
                failure: PhysicalRecoveryPublicationSettlementFailure::Signal(
                    PhysicalSignalSettlementOutcome::DerivedStateUnavailable
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
        PhysicalRecoveryPublicationCommandStage::RootProtocolReplacement,
    );
    assert_signal_stage(
        &outcome,
        PhysicalRecoveryPublicationCommandStage::RootProtocolReplacement,
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
        PhysicalRecoveryPublicationCommandStage::RecordNamespaceSynchronization,
    );
    assert_signal_stage(
        &outcome,
        PhysicalRecoveryPublicationCommandStage::RecordNamespaceSynchronization,
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

fn publication_signal_outcome(
    label: &str,
    stage: PhysicalRecoveryPublicationCommandStage,
) -> (PhysicalRecoveryPublicationIndeterminate, u64) {
    let retained_root = super::prepare_ordinary_recovery_root(label);
    let staged = super::selected_ordinary_recovery(retained_root.path())
        .plan()
        .unwrap()
        .stage()
        .unwrap();
    let expected_candidates = staged.publication_plan().candidates().len() as u64;
    staged.certification_fail_publication_signal_settlement_at(stage);
    let Err(PhysicalRecoveryOutcome::PublicationIndeterminate(outcome)) = staged.publish() else {
        panic!("publication Signal failure must retain indeterminate evidence")
    };
    (*outcome, expected_candidates)
}

fn assert_signal_stage(
    outcome: &PhysicalRecoveryPublicationIndeterminate,
    expected: PhysicalRecoveryPublicationCommandStage,
) {
    assert!(matches!(
        outcome.settlement().settlement(),
        PhysicalRecoveryPublicationSettlement::Indeterminate(
            PhysicalRecoveryPublicationCommandIndeterminate::Signal {
                stage,
                outcome: PhysicalSignalSettlementOutcome::DerivedStateUnavailable,
                ..
            }
        ) if *stage == expected
    ));
}

fn publication_fault_outcome(
    label: &str,
    role: MediaOperationRole,
    occurrence: u64,
    directive: MediaFaultDirective,
) -> PhysicalRecoveryPublicationIndeterminate {
    let retained_root = super::prepare_ordinary_recovery_root(label);
    let (selected, activation) = super::phase_six_publication::selected_with_publication_fault(
        retained_root.path(),
        role,
        occurrence,
        directive,
    );
    let staged = selected.plan().unwrap().stage().unwrap();
    activation.arm().unwrap();
    let Err(PhysicalRecoveryOutcome::PublicationIndeterminate(outcome)) = staged.publish() else {
        panic!("the activated publication fault must retain indeterminate evidence")
    };
    *outcome
}

fn assert_prefix(
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
