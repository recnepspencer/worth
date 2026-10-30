use crate::phase_three_media_denials::phase_three_support::synthetic_topology::publish_synthetic_nonempty_genesis;
#[allow(unused_imports)]
use crate::phase_three_media_denials::*;
use {
    std::path::Path, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis,
};
use {
    worth_store::physical_runtime::ArtifactTreeFailureKind,
    worth_store::physical_runtime::RecoveryDiscoveryArtifact,
};
use {
    worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily,
    worth_store_physical_format::RecordArtifactFile,
};
use {
    worth_store_physical_integrity::PhysicalDamageCause,
    worth_store_physical_integrity::PhysicalIntegrityRejection,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryBlockKind,
    worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryMediaObservationFailure,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial,
    worth_store_recovery_runtime::PhysicalRecoverySourceDenial,
};

#[test]
fn media_denials_retain_exact_fixed_artifact_and_backend_cause() {
    let selector_parent = tempfile::tempdir().unwrap();
    let selector_root = selector_parent.path().join("selector-directory");
    let selector_store = initialize_store(&selector_root);
    publish_synthetic_genesis(&selector_root, selector_store);
    replace_with_unreadable_entry(&current_selector(&selector_root));
    let selector_blocked = expect_blocked(
        admitted_recovery(&selector_root)
            .discover()
            .err()
            .expect("a directory in the current-selector slot must block"),
    );
    assert_media_denial(
        &selector_blocked,
        worth_store::physical_runtime::RecoveryDiscoveryArtifact::Record(
            worth_store_physical_format::RecordArtifactFile::CurrentRootSelector,
        ),
    );

    let checkpoint_parent = tempfile::tempdir().unwrap();
    let checkpoint_root = checkpoint_parent.path().join("checkpoint-directory");
    let checkpoint_store = initialize_store(&checkpoint_root);
    publish_synthetic_nonempty_genesis(&checkpoint_root, checkpoint_store);
    let records = checkpoint_root.join("families").join("records");
    let mut previous = std::fs::read(records.join("root-current.selector")).unwrap();
    previous[65] ^= 0x5a;
    std::fs::write(records.join("root-previous.selector"), previous).unwrap();
    let checkpoint = checkpoint_root.join("families").join("checkpoint.current");
    replace_absent_with_unreadable_entry(&checkpoint);
    let checkpoint_blocked = expect_blocked(
        admitted_recovery(&checkpoint_root)
            .discover()
            .err()
            .expect("a directory in the checkpoint slot must block"),
    );
    assert_media_denial(
        &checkpoint_blocked,
        worth_store::physical_runtime::RecoveryDiscoveryArtifact::CurrentCheckpoint,
    );
    assert!(matches!(
        checkpoint_blocked.evidence().integrity_observations(),
        [observation]
            if observation.scope().artifact_family()
                == worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootRoutingBlock
                && observation.outcome()
                    == worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome::Admitted
    ));
    assert!(checkpoint_blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(
                    worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
                ),
            } if localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch
        )));
}
