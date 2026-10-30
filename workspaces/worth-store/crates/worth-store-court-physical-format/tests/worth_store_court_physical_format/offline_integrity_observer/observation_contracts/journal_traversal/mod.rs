use crate::{
    phase_4_literal_vectors::wal_range, store_offline_integrity_observer_support::request,
    support::fixtures::clean_store,
};
use worth_store_offline_integrity_observer::observe_store;

#[test]
fn unaddressed_declared_block_does_not_hide_self_contained_damage_or_claim_parent_scope() {
    let fixture = clean_store("unaddressed-declared-block");
    let path = fixture
        .roots
        .join("root-000000000000000b-block-0000000000000003.manifest");
    let mut bytes = crate::phase_4_literal_vectors::routing_block();
    std::fs::write(&path, &bytes).unwrap();
    let clean = observe_store(&request(&fixture)).unwrap();
    let row = clean
        .artifacts()
        .iter()
        .find(|row| row.relative_path().ends_with("0000000000000003.manifest"))
        .unwrap();
    assert_eq!(
        row.family(),
        worth_foundational::PhysicalArtifactFamily::RootRoutingBlock
    );
    assert_eq!(
        row.outcome(),
        &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Unknown(worth_store_offline_integrity_observer::OfflineUnknownPhysicalReason::ParentScopeUnavailable)
    );
    bytes[44] ^= 1;
    std::fs::write(&path, bytes).unwrap();
    let damaged = observe_store(&request(&fixture)).unwrap();
    let row = damaged
        .artifacts()
        .iter()
        .find(|row| row.relative_path().ends_with("0000000000000003.manifest"))
        .unwrap();
    assert!(
        matches!(row.outcome(), worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Damaged(localization) if localization.cause() == worth_store_offline_integrity_observer::OfflinePhysicalDamageCause::ChecksumMismatch)
    );
}

#[test]
fn empty_canonical_wal_preserves_truncation_instead_of_disappearing_into_residue() {
    let fixture = clean_store("empty-wal");
    let wal = fixture.store.join("families/wal");
    std::fs::create_dir(&wal).unwrap();
    let path = wal.join("segment-1-generation-2.wal");
    std::fs::write(&path, wal_range(1, 3, 4)).unwrap();
    let clean = observe_store(&request(&fixture)).unwrap();
    assert!(clean.artifacts().iter().any(|row| row.family()
        == worth_foundational::PhysicalArtifactFamily::WalFrame
        && row.outcome()
            == &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Intact));
    std::fs::write(&path, []).unwrap();
    let damaged = observe_store(&request(&fixture)).unwrap();
    let rows: Vec<_> = damaged
        .artifacts()
        .iter()
        .filter(|row| row.relative_path().ends_with(".wal"))
        .collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].family(),
        worth_foundational::PhysicalArtifactFamily::WalFrame
    );
    assert!(
        matches!(rows[0].outcome(), worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Damaged(localization) if localization.cause() == worth_store_offline_integrity_observer::OfflinePhysicalDamageCause::Truncation)
    );
}

#[test]
fn adjacent_checksum_admitted_lsn_ranges_expose_only_required_interior_wal_gaps() {
    let fixture = clean_store("missing-middle-wal");
    let wal = fixture.store.join("families/wal");
    std::fs::create_dir(&wal).unwrap();
    for (segment, start) in [(1, 3), (2, 4), (10, 5)] {
        std::fs::write(
            wal.join(format!("segment-{segment}-generation-2.wal")),
            wal_range(segment, start, start + 1),
        )
        .unwrap();
    }
    let clean = observe_store(&request(&fixture)).unwrap();
    assert!(!clean
        .artifacts()
        .iter()
        .any(|row| row.identity().as_str().starts_with("wal-required-lsn:")));
    std::fs::remove_file(wal.join("segment-2-generation-2.wal")).unwrap();
    let missing = observe_store(&request(&fixture)).unwrap();
    let gaps: Vec<_> = missing
        .artifacts()
        .iter()
        .filter(|row| row.identity().as_str().starts_with("wal-required-lsn:"))
        .collect();
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].identity().as_str(), "wal-required-lsn:4:5");
    assert!(
        matches!(gaps[0].outcome(), worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Damaged(localization) if localization.cause() == worth_store_offline_integrity_observer::OfflinePhysicalDamageCause::MissingArtifact)
    );
    // Missing optional checkpoint and absent pre-retention WAL prefixes are not invented damage.
    assert!(!missing
        .artifacts()
        .iter()
        .any(|row| row.relative_path() == "families/checkpoint.current"));
    std::fs::write(wal.join("segment-11-generation-2.wal"), wal_range(11, 7, 8)).unwrap();
    let two_gaps = observe_store(&request(&fixture)).unwrap();
    use worth_store_offline_integrity_observer::{
        compare_integrity_observations, encode_offline_integrity_report,
        PhysicalIntegrityComparisonLimits,
    };
    let wire = encode_offline_integrity_report(&two_gaps).unwrap();
    let mut runtime: serde_json::Value = serde_json::from_str(&wire).unwrap();
    runtime["role"] = serde_json::json!("runtime-integrity-observer");
    runtime["process"] = serde_json::json!("other-process");
    runtime["executable"] = serde_json::json!("other-executable");
    // A comparison must be able to ingest every bounded absence row it emits.
    assert!(compare_integrity_observations(
        &runtime.to_string(),
        &wire,
        PhysicalIntegrityComparisonLimits::default()
    )
    .is_ok());
}
