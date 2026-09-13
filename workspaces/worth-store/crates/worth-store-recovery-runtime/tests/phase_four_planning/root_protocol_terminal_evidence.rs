use super::*;
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalIntegrityRejection};
use worth_store_recovery_runtime::{
    PhysicalRecoveryRootProtocolArtifact, PhysicalRecoveryRootProtocolDenial,
    PhysicalRecoverySourceDenial, SelectedPhysicalRecovery,
};

#[test]
fn damaged_previous_evidence_survives_every_post_selection_outcome() {
    let selected = selected_with_damaged_previous("root-protocol-progression");
    assert!(has_damaged_previous(selected.root_protocol_denials()));
    let planned = selected.plan().unwrap();
    assert!(has_damaged_previous(planned.root_protocol_denials()));
    let staged = planned.stage().unwrap();
    assert!(has_damaged_previous(staged.root_protocol_denials()));
    let durable = staged.publish().unwrap();
    assert!(has_damaged_previous(durable.root_protocol_denials()));
    let reopened = durable.reopen().unwrap();
    assert!(has_damaged_previous(reopened.root_protocol_denials()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("the unchanged secured basis must finish");
    };
    assert!(has_damaged_previous(handoff.root_protocol_denials()));
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn damaged_previous_evidence_survives_reopen_indeterminate() {
    let published = selected_with_damaged_previous("root-protocol-reopen-indeterminate")
        .plan()
        .unwrap()
        .stage()
        .unwrap()
        .publish()
        .unwrap();
    published.certification_fail_reopen_scheduler_settlement_at(
        worth_store::physical_runtime::PhysicalRecoveryFreshReopenStage::CurrentSelector,
    );
    let Err(PhysicalRecoveryOutcome::PublicationIndeterminate(outcome)) = published.reopen() else {
        panic!("the injected reopen settlement failure must be indeterminate");
    };
    assert!(has_damaged_previous(outcome.root_protocol_denials()));
}

fn selected_with_damaged_previous(label: &str) -> SelectedPhysicalRecovery {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.keep().join(label);
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_secured_synthetic_checkpoint(&root, store);
    let records = root.join("families").join("records");
    let mut previous = std::fs::read(records.join("root-current.selector")).unwrap();
    previous[65] ^= 0x5a;
    std::fs::write(records.join("root-previous.selector"), previous).unwrap();
    admitted_recovery(&root)
        .discover()
        .unwrap()
        .select()
        .unwrap()
}

fn has_damaged_previous(denials: &[PhysicalRecoverySourceDenial]) -> bool {
    denials.iter().any(|denial| {
        matches!(
            denial,
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(
                    PhysicalIntegrityRejection::Damaged(localization)
                ),
            } if localization.cause() == PhysicalDamageCause::ChecksumMismatch
        )
    })
}
