use crate::phase_three_root_anchor::phase_three_support::synthetic_topology::publish_synthetic_nonempty_genesis;
#[allow(unused_imports)]
use crate::phase_three_root_anchor::*;
use {
    std::path::Path, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis,
};
use {
    worth_store_physical_format::BootstrapCatalog,
    worth_store_physical_format::CurrentRootCatalogEntry,
    worth_store_physical_format::CurrentRootCatalogGeneration,
    worth_store_physical_format::DurableRootSelector,
    worth_store_physical_format::PhysicalRecordFormatDeclaration,
    worth_store_physical_format::RootSelectorIdentity,
    worth_store_physical_format::RootSelectorRole,
};
use {
    worth_store_physical_integrity::PhysicalDamageCause,
    worth_store_physical_integrity::PhysicalIntegrityRejection,
};
use {
    worth_store_recovery_physics::PhysicalRootCandidateDenial,
    worth_store_recovery_physics::PhysicalRootSelectionDenial,
    worth_store_recovery_physics::SelectedPhysicalRootRole,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial,
    worth_store_recovery_runtime::PhysicalRecoverySourceDenial,
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
        worth_store_recovery_physics::SelectedPhysicalRootRole::PreviousFallback
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
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(_),
            }
        )));
    assert_refusal_retains(
        selected.cancel_before_reconstruction(),
        worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
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
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(_),
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSlot {
                slot: worth_store_physical_format::RootSelectorRole::Current,
                denial: worth_store_recovery_physics::PhysicalRootCandidateDenial::SelectorIntegrity,
                ..
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSelection(
                worth_store_recovery_physics::PhysicalRootSelectionDenial::PreviousFallbackAnchorGenerationMismatch
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
    let original =
        worth_store_physical_format::DurableRootSelector::decode(&std::fs::read(&path).unwrap())
            .unwrap_err();
    assert!(matches!(
        original,
        worth_store_physical_format::RootSelectorDecodeDenial::Frame(_)
    ));
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let foreign = worth_store_physical_format::DurableRootSelector::new(
        foreign_store,
        format,
        worth_store_physical_format::RootSelectorIdentity::new(2).unwrap(),
        worth_store_physical_format::RootSelectorRole::Current,
        2,
        worth_store_physical_format::RootSelectorIdentity::new(1),
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
        worth_store_recovery_physics::SelectedPhysicalRootRole::PreviousFallback
    );
    assert!(selected
        .root_protocol_denials()
        .iter()
        .any(|denial| matches!(
            denial,
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(
                    worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
                ),
            } if localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::StoreIdentityMismatch
        )));
    assert_refusal_retains(
        selected.cancel_before_reconstruction(),
        worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
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
    assert_eq!(
        selected.root_role(),
        worth_store_recovery_physics::SelectedPhysicalRootRole::Current
    );
    assert!(selected
        .root_protocol_denials()
        .iter()
        .any(|denial| matches!(
            denial,
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(_),
            }
        )));
    assert_refusal_retains(
        selected.cancel_before_reconstruction(),
        worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
    );
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
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(
                    worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
                ),
            } if localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch
        )));
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::ManifestObservation(_)
        )));
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
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(
                    worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
                ),
            } if localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch
        )));
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::MediaObservation { .. }
        )));
}

/// Discovery's reader counts no reads of its own: each count the caller
/// admits is counted before the read it bounds. At the smallest counts a
/// caller can admit, both selectors, the previous root and the fallback
/// catalog are still read, and the previous root is selected.
#[test]
fn the_smallest_admitted_counts_still_read_the_selectors_and_the_fallback_catalog() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("exact-anchor");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_previous_and_anchor(&root, store, 2);
    // Discovery admits no fewer than the two selectors; one of every other
    // count it reads under.
    let mut declaration = limit_declaration(2, 1, 8 * 1024);
    declaration.manifest_entries = 1;
    declaration.checkpoint_candidates = 1;
    let selected = admitted_recovery_with_limits(
        &root,
        worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap(),
    )
    .discover()
    .unwrap()
    .select()
    .unwrap();
    assert_eq!(
        selected.root_role(),
        worth_store_recovery_physics::SelectedPhysicalRootRole::PreviousFallback
    );
    let counters = selected.discovery_counters();
    assert_eq!(counters.previous_selector_integrity_admissions, 1);
    assert_eq!(counters.previous_root_integrity_admissions, 1);
    assert_eq!(counters.bootstrap_integrity_admissions, 1);
    let _ = selected.cancel_before_reconstruction();
}
