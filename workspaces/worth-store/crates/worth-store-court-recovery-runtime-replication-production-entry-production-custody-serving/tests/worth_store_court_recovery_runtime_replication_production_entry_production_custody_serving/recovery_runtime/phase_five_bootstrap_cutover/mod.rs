#[allow(unused_imports)]
use crate::phase_five_bootstrap_cutover::*;
use {
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis,
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
        cause == worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch
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
        worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
            artifact:
                worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
            ..
        }
    )));
    assert_bootstrap_damage(denials, |cause| {
        cause == worth_store_physical_integrity::PhysicalDamageCause::StoreIdentityMismatch
    });
}

#[test]
fn missing_fallback_anchor_is_observed_without_changing_recovery_precedence() {
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
