#[allow(unused_imports)]
use crate::phase_three_root_protocol_denials::*;
use {
    crate::phase_three_support::admitted_recovery,
    crate::phase_three_support::admitted_recovery_for_format,
    crate::phase_three_support::expect_blocked, crate::phase_three_support::initialize_store,
    crate::phase_three_support::publish_synthetic_genesis,
    crate::phase_three_support::publish_synthetic_genesis_for_format, std::path::Path,
};
use {
    worth_store_physical_format::DurableRootSelector,
    worth_store_physical_format::PhysicalPageSizeClass,
    worth_store_physical_format::PhysicalRecordFormatDeclaration,
    worth_store_physical_format::RootSelectorRole,
    worth_store_physical_format::ROOT_SELECTOR_BYTES,
};
use {
    worth_store_physical_integrity::PhysicalDamageCause,
    worth_store_physical_integrity::PhysicalIntegrityRejection,
};
use {
    worth_store_recovery_physics::PhysicalRootCandidateDenial,
    worth_store_recovery_physics::PhysicalRootSelectionDenial,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryBlock,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial,
    worth_store_recovery_runtime::PhysicalRecoverySourceDenial,
};

#[test]
fn configured_supported_nondefault_format_admits_the_root_route() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("kib32-root");
    let store = initialize_store(&root);
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .page_size(worth_store_physical_format::PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    publish_synthetic_genesis_for_format(&root, store, format);

    let selected = admitted_recovery_for_format(&root, format)
        .discover()
        .unwrap()
        .select()
        .unwrap();
    assert_eq!(selected.root_generation(), 1);
    assert!(selected
        .root_protocol_denials()
        .iter()
        .any(|denial| matches!(
            denial,
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Absent,
            }
        )));
    let _ = selected.cancel_before_reconstruction();
}

#[test]
fn supported_but_misconfigured_selector_format_fails_closed() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("wrong-configured-format");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    let path = current_selector(&root);
    let selector =
        worth_store_physical_format::DurableRootSelector::decode(&std::fs::read(&path).unwrap())
            .unwrap();
    let other_format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .page_size(worth_store_physical_format::PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    let replacement = worth_store_physical_format::DurableRootSelector::new(
        selector.store_identity(),
        other_format,
        selector.identity(),
        selector.role(),
        selector.root_generation(),
        selector.linked_selector(),
        selector.linked_root_generation(),
    )
    .unwrap();
    std::fs::write(path, replacement.encode()).unwrap();

    let discovered = admitted_recovery(&root).discover().unwrap();
    let counters = discovered.counters();
    assert_eq!(counters.current_selector_integrity_admissions, 0);
    assert_eq!(counters.current_selector_interpretations, 0);
    assert_eq!(counters.current_root_integrity_admissions, 0);
    let blocked = expect_blocked(
        discovered
            .select()
            .err()
            .expect("the configured format mismatch must block"),
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
            } if localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::FormatMismatch
        )));
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSlot {
                slot: worth_store_physical_format::RootSelectorRole::Current,
                denial: worth_store_recovery_physics::PhysicalRootCandidateDenial::SelectorAuthorityMismatch,
                ..
            }
        )));
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSelection(
                worth_store_recovery_physics::PhysicalRootSelectionDenial::NoAdmittedRoot
            )
        )));
}

#[test]
fn poisoned_current_selector_stops_before_selector_or_root_interpretation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("poisoned-selector");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    poison_covered_byte(current_selector(&root), 65);

    let discovered = admitted_recovery(&root).discover().unwrap();
    let counters = discovered.counters();
    assert_eq!(counters.current_selector_integrity_admissions, 0);
    assert_eq!(counters.current_selector_interpretations, 0);
    assert_eq!(counters.current_root_integrity_admissions, 0);
    assert_eq!(counters.current_root_candidate_interpretations, 0);
    let blocked = expect_blocked(
        discovered
            .select()
            .err()
            .expect("the poisoned selector must block"),
    );
    assert!(matches!(
        blocked.evidence().source_denials.as_slice(),
        [
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(
                    worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
                ),
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Absent,
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSlot {
                slot: worth_store_physical_format::RootSelectorRole::Current,
                denial: worth_store_recovery_physics::PhysicalRootCandidateDenial::SelectorIntegrity,
                ..
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSelection(
                worth_store_recovery_physics::PhysicalRootSelectionDenial::NoAdmittedRoot
            )
        ] if localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch
    ));
}

#[test]
fn poisoned_addressed_root_stops_before_recovery_candidate_interpretation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("poisoned-root");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    poison_covered_byte(
        root.join("families")
            .join("records")
            .join("roots")
            .join("root-0000000000000001.manifest"),
        56,
    );

    let discovered = admitted_recovery(&root).discover().unwrap();
    let counters = discovered.counters();
    assert_eq!(counters.current_selector_integrity_admissions, 1);
    assert_eq!(counters.current_selector_interpretations, 1);
    assert_eq!(counters.current_root_integrity_admissions, 0);
    assert_eq!(counters.current_root_candidate_interpretations, 0);
    let blocked = expect_blocked(
        discovered
            .select()
            .err()
            .expect("the poisoned root must block"),
    );
    assert!(matches!(
        blocked.evidence().source_denials.as_slice(),
        [
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentRoot { generation: 1 },
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(
                    worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
                ),
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Absent,
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSlot {
                slot: worth_store_physical_format::RootSelectorRole::Current,
                denial: worth_store_recovery_physics::PhysicalRootCandidateDenial::RootIntegrity,
                observed_generation: Some(1),
                ..
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSelection(
                worth_store_recovery_physics::PhysicalRootSelectionDenial::CurrentRootRejected
            )
        ] if localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch
    ));
}

#[test]
fn root_denials_retain_wrong_role_missing_manifest_and_selector_format() {
    let wrong_role = root_case("wrong-role", |root| {
        let path = current_selector(root);
        let selector = worth_store_physical_format::DurableRootSelector::decode(
            &std::fs::read(&path).unwrap(),
        )
        .unwrap();
        let replacement = worth_store_physical_format::DurableRootSelector::new(
            selector.store_identity(),
            selector.format(),
            selector.identity(),
            worth_store_physical_format::RootSelectorRole::Previous,
            selector.root_generation(),
            selector.linked_selector(),
            selector.linked_root_generation(),
        )
        .unwrap();
        std::fs::write(path, replacement.encode()).unwrap();
    });
    assert!(matches!(
        wrong_role.evidence().source_denials.as_slice(),
        [
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(_),
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Absent,
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSlot {
                slot: worth_store_physical_format::RootSelectorRole::Current,
                denial: worth_store_recovery_physics::PhysicalRootCandidateDenial::SelectorAuthorityMismatch,
                observed_store: None,
                observed_role: None,
                observed_generation: None,
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSelection(
                worth_store_recovery_physics::PhysicalRootSelectionDenial::NoAdmittedRoot
            )
        ]
    ));

    let missing_manifest = root_case("missing-manifest", |root| {
        std::fs::remove_file(
            root.join("families")
                .join("records")
                .join("roots")
                .join("root-0000000000000001.manifest"),
        )
        .unwrap();
    });
    assert!(matches!(
        missing_manifest.evidence().source_denials.as_slice(),
        [
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentRoot { generation: 1 },
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Absent,
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Absent,
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSlot {
                slot: worth_store_physical_format::RootSelectorRole::Current,
                denial: worth_store_recovery_physics::PhysicalRootCandidateDenial::RootIntegrity,
                observed_generation: Some(1),
                ..
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSelection(
                worth_store_recovery_physics::PhysicalRootSelectionDenial::CurrentRootRejected
            )
        ]
    ));

    let selector_format = root_case("selector-format", |root| {
        std::fs::write(current_selector(root), [0_u8; ROOT_SELECTOR_BYTES]).unwrap();
    });
    assert!(matches!(
        selector_format.evidence().source_denials.as_slice(),
        [
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(_),
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Absent,
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSlot {
                slot: worth_store_physical_format::RootSelectorRole::Current,
                denial: worth_store_recovery_physics::PhysicalRootCandidateDenial::SelectorIntegrity,
                observed_store: None,
                observed_role: None,
                observed_generation: None,
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSelection(
                worth_store_recovery_physics::PhysicalRootSelectionDenial::NoAdmittedRoot
            )
        ]
    ));
}
