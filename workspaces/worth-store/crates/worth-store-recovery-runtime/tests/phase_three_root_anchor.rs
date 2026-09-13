#[allow(dead_code)]
mod phase_three_support;

use phase_three_support::*;
use worth_store_physical_format::{
    BootstrapCatalog, CurrentRootCatalogEntry, CurrentRootCatalogGeneration, DurableRootSelector,
    PhysicalRecordFormatDeclaration, RootSelectorIdentity, RootSelectorRole,
};
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalIntegrityRejection};
use worth_store_recovery_physics::{
    PhysicalRootCandidateDenial, PhysicalRootSelectionDenial, SelectedPhysicalRootRole,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryOutcome, PhysicalRecoveryRootProtocolArtifact,
    PhysicalRecoveryRootProtocolDenial, PhysicalRecoverySourceDenial,
};

#[test]
fn torn_current_fallback_requires_the_exact_completed_publication_anchor() {
    let exact_parent = tempfile::tempdir().unwrap();
    let exact_root = exact_parent.path().join("exact-anchor");
    let exact_store = initialize_store(&exact_root);
    publish_synthetic_genesis(&exact_root, exact_store);
    publish_previous_and_anchor(&exact_root, exact_store, 2);
    let selected = admitted_recovery(&exact_root)
        .discover()
        .unwrap()
        .select()
        .unwrap();
    assert_eq!(
        selected.root_role(),
        SelectedPhysicalRootRole::PreviousFallback
    );
    assert_eq!(selected.root_generation(), 1);
    let counters = selected.discovery_counters();
    assert_eq!(counters.current_selector_integrity_admissions, 0);
    assert_eq!(counters.current_selector_interpretations, 0);
    assert_eq!(counters.previous_selector_integrity_admissions, 1);
    assert_eq!(counters.previous_selector_interpretations, 1);
    assert_eq!(counters.previous_root_integrity_admissions, 1);
    assert_eq!(counters.previous_root_candidate_interpretations, 1);
    assert!(selected
        .root_protocol_denials()
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(_),
            }
        )));
    assert_refusal_retains(
        selected.cancel_before_reconstruction(),
        PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
    );

    let stale_parent = tempfile::tempdir().unwrap();
    let stale_root = stale_parent.path().join("stale-anchor");
    let stale_store = initialize_store(&stale_root);
    publish_synthetic_genesis(&stale_root, stale_store);
    publish_previous_and_anchor(&stale_root, stale_store, 3);
    let blocked = expect_blocked(
        admitted_recovery(&stale_root)
            .discover()
            .unwrap()
            .select()
            .err()
            .expect("a stale publication anchor must block previous fallback"),
    );
    assert!(matches!(
        blocked.evidence().source_denials.as_slice(),
        [
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(_),
            },
            PhysicalRecoverySourceDenial::RootSlot {
                slot: RootSelectorRole::Current,
                denial: PhysicalRootCandidateDenial::SelectorIntegrity,
                ..
            },
            PhysicalRecoverySourceDenial::RootSelection(
                PhysicalRootSelectionDenial::PreviousFallbackAnchorGenerationMismatch
            )
        ]
    ));
}

#[test]
fn authority_mismatched_current_preserves_exact_anchored_previous_fallback() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("authority-mismatch-fallback");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_previous_and_anchor(&root, store, 2);
    let foreign_root = parent.path().join("foreign");
    let foreign_store = initialize_store(&foreign_root);
    let path = root
        .join("families")
        .join("records")
        .join("root-current.selector");
    let original = DurableRootSelector::decode(&std::fs::read(&path).unwrap()).unwrap_err();
    assert!(matches!(
        original,
        worth_store_physical_format::RootSelectorDecodeDenial::Frame(_)
    ));
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let foreign = DurableRootSelector::new(
        foreign_store,
        format,
        RootSelectorIdentity::new(2).unwrap(),
        RootSelectorRole::Current,
        2,
        RootSelectorIdentity::new(1),
        Some(1),
    )
    .unwrap();
    std::fs::write(path, foreign.encode()).unwrap();

    let selected = admitted_recovery(&root)
        .discover()
        .unwrap()
        .select()
        .unwrap();
    assert_eq!(
        selected.root_role(),
        SelectedPhysicalRootRole::PreviousFallback
    );
    assert!(selected
        .root_protocol_denials()
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(
                    PhysicalIntegrityRejection::Damaged(localization)
                ),
            } if localization.cause() == PhysicalDamageCause::StoreIdentityMismatch
        )));
    assert_refusal_retains(
        selected.cancel_before_reconstruction(),
        PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
    );
}

#[test]
fn valid_current_retains_a_damaged_previous_observation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("damaged-previous");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    let records = root.join("families").join("records");
    let mut previous = std::fs::read(records.join("root-current.selector")).unwrap();
    previous[65] ^= 0x5a;
    std::fs::write(records.join("root-previous.selector"), previous).unwrap();

    let discovered = admitted_recovery(&root).discover().unwrap();
    let counters = discovered.counters();
    assert_eq!(counters.current_selector_integrity_admissions, 1);
    assert_eq!(counters.current_selector_interpretations, 1);
    assert_eq!(counters.current_root_integrity_admissions, 1);
    assert_eq!(counters.current_root_candidate_interpretations, 1);
    assert_eq!(counters.previous_selector_integrity_admissions, 0);
    assert_eq!(counters.previous_selector_interpretations, 0);
    assert_eq!(counters.previous_root_integrity_admissions, 0);
    assert_eq!(counters.previous_root_candidate_interpretations, 0);
    let selected = discovered.select().unwrap();
    assert_eq!(selected.root_role(), SelectedPhysicalRootRole::Current);
    assert!(selected
        .root_protocol_denials()
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(_),
            }
        )));
    assert_refusal_retains(
        selected.cancel_before_reconstruction(),
        PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
    );
}

fn assert_refusal_retains(
    outcome: PhysicalRecoveryOutcome,
    artifact: PhysicalRecoveryRootProtocolArtifact,
) {
    let PhysicalRecoveryOutcome::Refused(refusal) = outcome else {
        panic!("pre-reconstruction cancellation must refuse recovery");
    };
    assert!(refusal
        .root_protocol_denials()
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: observed,
                ..
            } if *observed == artifact
        )));
}

#[test]
fn later_selection_failure_retains_the_damaged_previous_observation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("damaged-previous-and-manifest-failure");
    let store = initialize_store(&root);
    publish_synthetic_nonempty_genesis(&root, store);
    let records = root.join("families").join("records");
    let mut previous = std::fs::read(records.join("root-current.selector")).unwrap();
    previous[65] ^= 0x5a;
    std::fs::write(records.join("root-previous.selector"), previous).unwrap();
    std::fs::remove_file(
        records
            .join("roots")
            .join("root-0000000000000001-block-0000000000000001.manifest"),
    )
    .unwrap();

    let blocked = expect_blocked(
        admitted_recovery(&root)
            .discover()
            .unwrap()
            .select()
            .err()
            .expect("the missing routing block must block selection"),
    );
    assert!(blocked
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
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(denial, PhysicalRecoverySourceDenial::ManifestObservation(_))));
}

#[test]
fn damaged_current_survives_a_later_previous_slot_media_failure() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent
        .path()
        .join("damaged-current-and-previous-media-failure");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    let records = root.join("families").join("records");
    let current_path = records.join("root-current.selector");
    let mut current = std::fs::read(&current_path).unwrap();
    current[65] ^= 0x5a;
    std::fs::write(current_path, current).unwrap();
    std::fs::create_dir(records.join("root-previous.selector")).unwrap();

    let blocked = expect_blocked(
        admitted_recovery(&root)
            .discover()
            .err()
            .expect("the nonregular previous slot must fail discovery"),
    );
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(
                    PhysicalIntegrityRejection::Damaged(localization)
                ),
            } if localization.cause() == PhysicalDamageCause::ChecksumMismatch
        )));
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::MediaObservation { .. }
        )));
}

fn publish_previous_and_anchor(
    root: &std::path::Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    anchor_generation: u64,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let previous = DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(1).unwrap(),
        RootSelectorRole::Previous,
        1,
        RootSelectorIdentity::new(2),
        Some(2),
    )
    .unwrap();
    let records = root.join("families").join("records");
    std::fs::write(records.join("root-previous.selector"), previous.encode()).unwrap();
    let catalog = BootstrapCatalog::new(
        store,
        format,
        CurrentRootCatalogEntry::new(CurrentRootCatalogGeneration::new(anchor_generation).unwrap()),
    );
    std::fs::write(records.join("bootstrap.catalog"), catalog.encode()).unwrap();
    let current_path = records.join("root-current.selector");
    let mut current = std::fs::read(&current_path).unwrap();
    current[65] ^= 0x5a;
    std::fs::write(current_path, current).unwrap();
}
