#[allow(unused_imports)] use crate::phase_four_planning::root_protocol_terminal_evidence::*;
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery, worth_store_testkit_recovery_runtime::phase_three_support::initialize_store, crate::phase_four_planning::publish_secured_synthetic_checkpoint, worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_covered_wal, worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis};
use {worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection};
use {worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact, worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial, worth_store_recovery_runtime::PhysicalRecoverySourceDenial, worth_store_recovery_runtime::SelectedPhysicalRecovery};

#[test]
fn damaged_previous_evidence_survives_every_post_selection_outcome() {
    let (_world, selected) = selected_with_damaged_previous("root-protocol-progression");
    assert!(has_damaged_previous(selected.root_protocol_denials()));
    let planned = selected.plan().unwrap();
    assert!(has_damaged_previous(planned.root_protocol_denials()));
    let staged = planned.stage().unwrap();
    assert!(has_damaged_previous(staged.root_protocol_denials()));
    let durable = staged.publish().unwrap();
    assert!(has_damaged_previous(durable.root_protocol_denials()));
    let reopened = durable.reopen().unwrap();
    assert!(has_damaged_previous(reopened.root_protocol_denials()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("the unchanged secured basis must finish");
    };
    assert!(has_damaged_previous(handoff.root_protocol_denials()));
}

#[cfg(all())]
#[test]
fn damaged_previous_evidence_survives_reopen_indeterminate() {
    let (_world, selected) = selected_with_damaged_previous("root-protocol-reopen-indeterminate");
    let published = selected.plan().unwrap().stage().unwrap().publish().unwrap();
    published.certification_fail_reopen_scheduler_settlement_at(
        worth_store::physical_runtime::PhysicalRecoveryFreshReopenStage::CurrentSelector,
    );
    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(outcome)) = published.reopen() else {
        panic!("the injected reopen settlement failure must be indeterminate");
    };
    assert!(has_damaged_previous(outcome.root_protocol_denials()));
}
