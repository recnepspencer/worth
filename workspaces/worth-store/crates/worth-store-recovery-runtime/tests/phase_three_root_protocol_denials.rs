#[allow(dead_code)]
#[path = "phase_three_support/mod.rs"]
mod phase_three_support;

use self::phase_three_support::*;
use worth_store_physical_format::{
    DurableRootSelector, PhysicalPageSizeClass, PhysicalRecordFormatDeclaration, RootSelectorRole,
    ROOT_SELECTOR_BYTES,
};
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalIntegrityRejection};
use worth_store_recovery_physics::{PhysicalRootCandidateDenial, PhysicalRootSelectionDenial};
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlock, PhysicalRecoveryRootProtocolArtifact,
    PhysicalRecoveryRootProtocolDenial, PhysicalRecoverySourceDenial,
};

#[test]
fn configured_supported_nondefault_format_admits_the_root_route() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("kib32-root");
    let store = initialize_store(&root);
    let format = PhysicalRecordFormatDeclaration::builder()
        .page_size(PhysicalPageSizeClass::KiB32)
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
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Absent,
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
    let selector = DurableRootSelector::decode(&std::fs::read(&path).unwrap()).unwrap();
    let other_format = PhysicalRecordFormatDeclaration::builder()
        .page_size(PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    let replacement = DurableRootSelector::new(
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
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(
                    PhysicalIntegrityRejection::Damaged(localization)
                ),
            } if localization.cause() == PhysicalDamageCause::FormatMismatch
        )));
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::RootSlot {
                slot: RootSelectorRole::Current,
                denial: PhysicalRootCandidateDenial::SelectorAuthorityMismatch,
                ..
            }
        )));
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::RootSelection(
                PhysicalRootSelectionDenial::NoAdmittedRoot
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
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(
                    PhysicalIntegrityRejection::Damaged(localization)
                ),
            },
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Absent,
            },
            PhysicalRecoverySourceDenial::RootSlot {
                slot: RootSelectorRole::Current,
                denial: PhysicalRootCandidateDenial::SelectorIntegrity,
                ..
            },
            PhysicalRecoverySourceDenial::RootSelection(
                PhysicalRootSelectionDenial::NoAdmittedRoot
            )
        ] if localization.cause() == PhysicalDamageCause::ChecksumMismatch
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
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CurrentRoot { generation: 1 },
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(
                    PhysicalIntegrityRejection::Damaged(localization)
                ),
            },
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Absent,
            },
            PhysicalRecoverySourceDenial::RootSlot {
                slot: RootSelectorRole::Current,
                denial: PhysicalRootCandidateDenial::RootIntegrity,
                observed_generation: Some(1),
                ..
            },
            PhysicalRecoverySourceDenial::RootSelection(
                PhysicalRootSelectionDenial::CurrentRootRejected
            )
        ] if localization.cause() == PhysicalDamageCause::ChecksumMismatch
    ));
}

#[test]
fn root_denials_retain_wrong_role_missing_manifest_and_selector_format() {
    let wrong_role = root_case("wrong-role", |root| {
        let path = current_selector(root);
        let selector = DurableRootSelector::decode(&std::fs::read(&path).unwrap()).unwrap();
        let replacement = DurableRootSelector::new(
            selector.store_identity(),
            selector.format(),
            selector.identity(),
            RootSelectorRole::Previous,
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
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(_),
            },
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Absent,
            },
            PhysicalRecoverySourceDenial::RootSlot {
                slot: RootSelectorRole::Current,
                denial: PhysicalRootCandidateDenial::SelectorAuthorityMismatch,
                observed_store: None,
                observed_role: None,
                observed_generation: None,
            },
            PhysicalRecoverySourceDenial::RootSelection(
                PhysicalRootSelectionDenial::NoAdmittedRoot
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
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CurrentRoot { generation: 1 },
                denial: PhysicalRecoveryRootProtocolDenial::Absent,
            },
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Absent,
            },
            PhysicalRecoverySourceDenial::RootSlot {
                slot: RootSelectorRole::Current,
                denial: PhysicalRootCandidateDenial::RootIntegrity,
                observed_generation: Some(1),
                ..
            },
            PhysicalRecoverySourceDenial::RootSelection(
                PhysicalRootSelectionDenial::CurrentRootRejected
            )
        ]
    ));

    let selector_format = root_case("selector-format", |root| {
        std::fs::write(current_selector(root), [0_u8; ROOT_SELECTOR_BYTES]).unwrap();
    });
    assert!(matches!(
        selector_format.evidence().source_denials.as_slice(),
        [
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(_),
            },
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Absent,
            },
            PhysicalRecoverySourceDenial::RootSlot {
                slot: RootSelectorRole::Current,
                denial: PhysicalRootCandidateDenial::SelectorIntegrity,
                observed_store: None,
                observed_role: None,
                observed_generation: None,
            },
            PhysicalRecoverySourceDenial::RootSelection(
                PhysicalRootSelectionDenial::NoAdmittedRoot
            )
        ]
    ));
}

fn root_case(name: &str, mutate: impl FnOnce(&std::path::Path)) -> PhysicalRecoveryBlock {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join(name);
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    mutate(&root);
    expect_blocked(
        admitted_recovery(&root)
            .discover()
            .unwrap()
            .select()
            .err()
            .expect("root denial must block"),
    )
}

fn current_selector(root: &std::path::Path) -> std::path::PathBuf {
    root.join("families")
        .join("records")
        .join("root-current.selector")
}

fn poison_covered_byte(path: std::path::PathBuf, offset: usize) {
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[offset] ^= 0x5a;
    std::fs::write(path, bytes).unwrap();
}
