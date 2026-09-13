#[allow(dead_code)]
mod phase_three_support;

use phase_three_support::*;
use worth_store::physical_runtime::{ArtifactTreeFailureKind, RecoveryDiscoveryArtifact};
use worth_store_physical_format::{
    integrity_declarations::PhysicalIntegrityArtifactFamily, RecordArtifactFile,
};
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalIntegrityRejection};
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryIntegrityObservationOutcome,
    PhysicalRecoveryMediaObservationFailure, PhysicalRecoveryRootProtocolArtifact,
    PhysicalRecoveryRootProtocolDenial, PhysicalRecoverySourceDenial,
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
        RecoveryDiscoveryArtifact::Record(RecordArtifactFile::CurrentRootSelector),
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
        RecoveryDiscoveryArtifact::CurrentCheckpoint,
    );
    assert!(matches!(
        checkpoint_blocked.evidence().integrity_observations(),
        [observation]
            if observation.scope().artifact_family()
                == PhysicalIntegrityArtifactFamily::RootRoutingBlock
                && observation.outcome()
                    == PhysicalRecoveryIntegrityObservationOutcome::Admitted
    ));
    assert!(checkpoint_blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(
                    PhysicalIntegrityRejection::Damaged(localization)
                ),
            } if localization.cause() == PhysicalDamageCause::ChecksumMismatch
        )));
}

fn assert_media_denial(
    blocked: &worth_store_recovery_runtime::PhysicalRecoveryBlock,
    expected_artifact: RecoveryDiscoveryArtifact,
) {
    assert_eq!(blocked.kind, PhysicalRecoveryBlockKind::MediaObservation);
    let Some((artifact, failure)) =
        blocked
            .evidence()
            .source_denials
            .iter()
            .find_map(|denial| match denial {
                PhysicalRecoverySourceDenial::MediaObservation { artifact, failure } => {
                    Some((artifact, failure))
                }
                _ => None,
            })
    else {
        panic!("media failure must retain one exact typed denial")
    };
    assert_eq!(artifact, &expected_artifact);
    assert!(
        matches!(
            failure,
            PhysicalRecoveryMediaObservationFailure::Backend {
                kind: ArtifactTreeFailureKind::DeniedBeforeEffect,
                io_kind: Some(_),
            }
        ),
        "unexpected media failure: {failure:?}"
    );
    assert_eq!(blocked.recovery_effects(), 0);
}

fn current_selector(root: &std::path::Path) -> std::path::PathBuf {
    root.join("families")
        .join("records")
        .join("root-current.selector")
}

fn replace_with_unreadable_entry(path: &std::path::Path) {
    std::fs::remove_file(path).unwrap();
    replace_absent_with_unreadable_entry(path);
}

#[cfg(windows)]
fn replace_absent_with_unreadable_entry(path: &std::path::Path) {
    std::fs::create_dir(path).unwrap();
}

#[cfg(unix)]
fn replace_absent_with_unreadable_entry(path: &std::path::Path) {
    std::os::unix::fs::symlink(path.file_name().unwrap(), path).unwrap();
}
