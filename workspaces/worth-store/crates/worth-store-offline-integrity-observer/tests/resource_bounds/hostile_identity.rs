use super::*;

#[test]
fn hard_link_alias_is_reported_twice_but_read_once() {
    let fixture = clean_store("hard-link-alias");
    let source = fixture.records.join("root-current.selector");
    let alias = fixture
        .records
        .join("root-current-0000000000000001.candidate");
    std::fs::hard_link(&source, &alias).expect("same-volume hard-link fixture must be supported");
    let report = observe_store(&bounded_request(&fixture, limits(100, 16 * 1024, 8, 0))).unwrap();
    let duplicates: Vec<_> = report
        .artifacts()
        .iter()
        .filter(|artifact| {
            artifact.family() == PhysicalArtifactFamily::CurrentRootSelector
                && artifact.duplicates().iter().any(|evidence| {
                    matches!(
                        evidence,
                        OfflineArtifactDuplicateEvidence::PhysicalAlias { .. }
                    )
                })
        })
        .collect();
    assert_eq!(duplicates.len(), 1);
    assert!(duplicates
        .iter()
        .all(|artifact| artifact.outcome() == &OfflineIntegrityOutcome::Unknown(
            worth_store_offline_integrity_observer::OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected)));
    assert!(matches!(duplicates[0].duplicates(),
        [OfflineArtifactDuplicateEvidence::PhysicalAlias { first_path }]
        if &**first_path == "families/records/root-current.selector"));
    assert_eq!(report.counters().checksum_calculations(), 4);
    assert_eq!(report.counters().selector_payload_decoder_entries(), 2);
    assert_eq!(report.counters().duplicate_identities(), 1);
    assert_eq!(report.counters().bytes_read(), 654);
    assert_eq!(report.counters().files_opened(), 20);
}

#[test]
fn hard_link_across_protocol_scopes_is_still_read_once() {
    let fixture = clean_store("cross-scope-hard-link");
    let source = fixture.records.join("root-current.selector");
    let root = fixture.roots.join("root-0000000000000001.manifest");
    std::fs::remove_file(&root).unwrap();
    std::fs::hard_link(&source, &root).expect("same-volume hard-link fixture must be supported");
    let report = observe_store(&bounded_request(&fixture, limits(100, 16 * 1024, 8, 0))).unwrap();
    assert_eq!(report.counters().bytes_read(), 286);
    assert_eq!(report.counters().files_opened(), 16);
    let root = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.family() == PhysicalArtifactFamily::RootManifest)
        .unwrap();
    assert_eq!(root.outcome(), &OfflineIntegrityOutcome::Unknown(
        worth_store_offline_integrity_observer::OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected));
    assert_eq!(report.counters().root_manifest_payload_decoder_entries(), 0);
    assert_eq!(report.counters().checksum_calculations(), 3);
}

#[test]
fn unknown_entry_is_visible_and_observation_does_not_mutate_store() {
    let fixture = clean_store("unknown-read-only");
    std::fs::write(fixture.records.join("mystery.record"), b"opaque").unwrap();
    let before = store_snapshot(&fixture.store);
    let report = observe_store(&bounded_request(&fixture, limits(100, 16 * 1024, 8, 0))).unwrap();
    let unknown = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.relative_path() == "families/records/mystery.record")
        .unwrap();
    assert_eq!(unknown.family(), OfflineArtifactFamily::Unrecognized);
    assert_eq!(
        unknown.outcome(),
        &OfflineIntegrityOutcome::Unknown(
            worth_store_offline_integrity_observer::OfflineUnknownPhysicalReason::UnrecognizedFile
        )
    );
    assert_eq!(store_snapshot(&fixture.store), before);
    assert!(!fixture.report.exists());
}

#[test]
fn unknown_hard_link_is_visible_as_a_physical_alias_without_reinspection() {
    let fixture = clean_store("unknown-hard-link");
    let source = fixture.records.join("root-current.selector");
    let alias = fixture.records.join("mystery.record");
    std::fs::hard_link(&source, &alias).expect("same-volume hard-link fixture must be supported");
    let report = observe_store(&bounded_request(&fixture, limits(100, 16 * 1024, 8, 0))).unwrap();
    let unknown = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.relative_path() == "families/records/mystery.record")
        .unwrap();
    assert_eq!(
        unknown.outcome(),
        &OfflineIntegrityOutcome::Unknown(
            worth_store_offline_integrity_observer::OfflineUnknownPhysicalReason::UnrecognizedFile
        )
    );
    assert!(matches!(
        unknown.duplicates(),
        [OfflineArtifactDuplicateEvidence::PhysicalAlias { first_path }]
            if &**first_path == "families/records/root-current.selector"
    ));
    assert_eq!(report.counters().bytes_read(), 654);
    assert_eq!(report.counters().duplicate_identities(), 1);
}

#[test]
fn unknown_directory_is_classified_without_being_traversed() {
    let fixture = clean_store("unknown-directory");
    let directory = fixture.records.join("mystery.directory");
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("hidden"), vec![0x5a; 4_096]).unwrap();
    let report = observe_store(&bounded_request(&fixture, limits(100, 16 * 1024, 8, 0))).unwrap();
    let unknown = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.relative_path() == "families/records/mystery.directory")
        .unwrap();
    assert_eq!(
        unknown.outcome(),
        &OfflineIntegrityOutcome::Unknown(
            worth_store_offline_integrity_observer::OfflineUnknownPhysicalReason::UnrecognizedDirectory
        )
    );
    assert!(report
        .artifacts()
        .iter()
        .all(|artifact| !artifact.relative_path().contains("hidden")));
    assert_eq!(report.counters().bytes_read(), 654);
}
