#[allow(dead_code)]
mod phase_three_support;

use phase_three_support::*;
use worth_store_physical_format::{
    BootstrapCatalog, CurrentRootCatalogEntry, CurrentRootCatalogGeneration, DurableRootSelector,
    PhysicalRecordFormatDeclaration, RootSelectorIdentity, RootSelectorRole,
};
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalIntegrityRejection};
use worth_store_recovery_runtime::{
    PhysicalRecoveryRootProtocolArtifact, PhysicalRecoveryRootProtocolDenial,
    PhysicalRecoverySourceDenial,
};

#[test]
fn corrupt_bootstrap_is_rejected_before_owner_projection() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("corrupt-bootstrap");
    let store = initialize_store(&root);
    prepare_previous_fallback(&root, store);
    publish_catalog(&root, store);
    let catalog_path = records(&root).join("bootstrap.catalog");
    let mut bytes = std::fs::read(&catalog_path).unwrap();
    bytes[44] ^= 0x80;
    std::fs::write(catalog_path, bytes).unwrap();

    let discovered = admitted_recovery(&root).discover().unwrap();
    assert_rejected_before_projection(discovered.counters(), false);
    let blocked = expect_blocked(
        discovered
            .select()
            .err()
            .expect("corrupt bootstrap must block fallback selection"),
    );
    assert_bootstrap_damage(blocked.evidence().source_denials.as_slice(), |cause| {
        cause == PhysicalDamageCause::ChecksumMismatch
    });
    let observations = blocked.evidence().integrity_observations();
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].scope().store_identity(), store);
    assert_eq!(observations[0].scope().artifact_family(), worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::BootstrapCatalog);
    assert!(matches!(
        observations[0].outcome(),
        worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome::Rejected(_)
    ));
}

#[test]
fn wrong_store_bootstrap_is_rejected_before_owner_projection() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("wrong-store-bootstrap");
    let store = initialize_store(&root);
    prepare_previous_fallback(&root, store);
    let foreign_root = parent.path().join("foreign");
    let foreign_store = initialize_store(&foreign_root);
    publish_catalog(&root, foreign_store);

    let discovered = admitted_recovery(&root).discover().unwrap();
    assert_rejected_before_projection(discovered.counters(), false);
    let blocked = expect_blocked(
        discovered
            .select()
            .err()
            .expect("wrong-store bootstrap must block fallback selection"),
    );
    let denials = blocked.evidence().source_denials.as_slice();
    assert!(denials.iter().any(|denial| matches!(
        denial,
        PhysicalRecoverySourceDenial::RootProtocol {
            artifact: PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
            ..
        }
    )));
    assert_bootstrap_damage(denials, |cause| {
        cause == PhysicalDamageCause::StoreIdentityMismatch
    });
}

#[test]
fn missing_fallback_anchor_is_observed_without_changing_c8_precedence() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("absent-bootstrap");
    let store = initialize_store(&root);
    prepare_previous_fallback(&root, store);

    let discovered = admitted_recovery(&root).discover().unwrap();
    let counters = discovered.counters();
    assert_eq!(counters.bootstrap_integrity_attempts, 1);
    assert_eq!(counters.bootstrap_absent, 1);
    assert_eq!(counters.bootstrap_integrity_rejections, 0);
    assert_eq!(counters.bootstrap_owner_projections, 0);
    assert_eq!(counters.bootstrap_owner_decoder_entries, 0);
    let _ = expect_blocked(
        discovered
            .select()
            .err()
            .expect("absent bootstrap must block fallback selection"),
    );

    let current_root = parent.path().join("valid-current");
    let current_store = initialize_store(&current_root);
    publish_synthetic_genesis(&current_root, current_store);
    let selected = admitted_recovery(&current_root).discover().unwrap();
    assert_eq!(selected.counters().bootstrap_integrity_attempts, 0);
    selected.select().unwrap();
}

fn assert_rejected_before_projection(
    counters: worth_store_recovery_runtime::PhysicalRecoveryDiscoveryCounters,
    absent: bool,
) {
    assert_eq!(counters.bootstrap_integrity_attempts, 1);
    assert_eq!(counters.bootstrap_integrity_admissions, 0);
    assert_eq!(counters.bootstrap_integrity_rejections, u64::from(!absent));
    assert_eq!(counters.bootstrap_owner_projections, 0);
    assert_eq!(counters.bootstrap_owner_decoder_entries, 0);
}

fn assert_bootstrap_damage(
    denials: &[PhysicalRecoverySourceDenial],
    expected: impl Fn(PhysicalDamageCause) -> bool,
) {
    assert!(denials.iter().any(|denial| matches!(
        denial,
        PhysicalRecoverySourceDenial::RootProtocol {
            artifact: PhysicalRecoveryRootProtocolArtifact::BootstrapCatalog,
            denial: PhysicalRecoveryRootProtocolDenial::Integrity(
                PhysicalIntegrityRejection::Damaged(localization)
            ),
        } if expected(localization.cause())
    )));
}

fn prepare_previous_fallback(
    root: &std::path::Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) {
    publish_synthetic_genesis(root, store);
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
    std::fs::write(
        records(root).join("root-previous.selector"),
        previous.encode(),
    )
    .unwrap();
    let current_path = records(root).join("root-current.selector");
    let mut current = std::fs::read(&current_path).unwrap();
    current[65] ^= 0x5a;
    std::fs::write(current_path, current).unwrap();
}

fn publish_catalog(
    root: &std::path::Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) {
    let catalog = BootstrapCatalog::new(
        store,
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        CurrentRootCatalogEntry::new(CurrentRootCatalogGeneration::new(2).unwrap()),
    );
    std::fs::write(records(root).join("bootstrap.catalog"), catalog.encode()).unwrap();
}

fn records(root: &std::path::Path) -> std::path::PathBuf {
    root.join("families").join("records")
}
