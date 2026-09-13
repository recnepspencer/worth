use std::fs;

use worth_foundational::PhysicalIntegrityPosture;
use worth_store_offline_integrity_observer::{
    observe_store, OfflineArtifactObservation, OfflineIntegrityOutcome, OfflinePhysicalDamageCause,
    OfflinePhysicalFormatField, OfflineUnknownPhysicalReason,
};

use crate::support::{
    artifact_path, clean_store, current_selector_bytes, refresh_crc32c, request, Target,
};

#[test]
fn checksum_invalid_root_cannot_forge_duplicate_identity() {
    let fixture = clean_store("forged-root-duplicate");
    let mut forged = fs::read(artifact_path(&fixture, Target::Root)).unwrap();
    forged[44] ^= 1;
    fs::write(fixture.roots.join("root-0000000000000002.manifest"), forged).unwrap();
    let report = observe_store(&request(&fixture)).unwrap();
    assert_eq!(report.counters().duplicate_identities(), 0);
    assert_eq!(
        target_observation(&report, Target::Root)
            .outcome()
            .posture(),
        PhysicalIntegrityPosture::Intact
    );
    assert!(report.artifacts().iter().all(|artifact| {
        !matches!(
            artifact.outcome(),
            OfflineIntegrityOutcome::Damaged(localization)
                if localization.cause() == OfflinePhysicalDamageCause::DuplicateIdentity
        )
    }));
    let forged = report
        .artifacts()
        .iter()
        .find(|artifact| {
            artifact.relative_path() == "families/records/roots/root-0000000000000002.manifest"
        })
        .expect("forged candidate remains visible");
    let OfflineIntegrityOutcome::Damaged(localization) = forged.outcome() else {
        panic!("forged candidate must retain checksum damage");
    };
    assert_eq!(
        localization.cause(),
        OfflinePhysicalDamageCause::ChecksumMismatch
    );
    assert_eq!(
        localization
            .damaged_range()
            .map(|range| (range.offset(), range.length())),
        Some((0, 368))
    );
}

#[test]
fn damaged_noncanonical_selector_candidate_remains_visible() {
    let fixture = clean_store("damaged-selector-candidate");
    let mut candidate = current_selector_bytes();
    candidate[44] ^= 1;
    let relative = "families/records/root-current-0000000000000002.candidate";
    fs::write(fixture.store.join(relative), candidate).unwrap();
    let report = observe_store(&request(&fixture)).unwrap();
    let observation = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.relative_path() == relative)
        .expect("recognized candidate remains visible");
    let OfflineIntegrityOutcome::Damaged(localization) = observation.outcome() else {
        panic!("candidate must retain checksum damage");
    };
    assert_eq!(
        localization.cause(),
        OfflinePhysicalDamageCause::ChecksumMismatch
    );
}

#[test]
fn foreign_store_candidate_cannot_poison_canonical_duplicate_identity() {
    let fixture = clean_store("foreign-selector-candidate");
    let mut candidate = current_selector_bytes();
    candidate[48..64].copy_from_slice(&[0xa5; 16]);
    refresh_crc32c(&mut candidate);
    fs::write(
        fixture
            .records
            .join("root-current-0000000000000002.candidate"),
        candidate,
    )
    .unwrap();
    let report = observe_store(&request(&fixture)).unwrap();
    assert_eq!(report.counters().duplicate_identities(), 0);
    assert_eq!(
        target_observation(&report, Target::Current).outcome(),
        &OfflineIntegrityOutcome::Intact
    );
    let foreign = report
        .artifacts()
        .iter()
        .find(|artifact| {
            artifact
                .relative_path()
                .ends_with("0000000000000002.candidate")
        })
        .unwrap();
    let OfflineIntegrityOutcome::Damaged(localization) = foreign.outcome() else {
        panic!("foreign candidate must be scoped out");
    };
    assert_eq!(
        localization.cause(),
        OfflinePhysicalDamageCause::ScopeMismatch
    );
    assert_eq!(
        localization.field(),
        Some(OfflinePhysicalFormatField::StoreIdentity)
    );
}

#[test]
fn checksum_valid_unaddressed_root_is_unknown_and_not_a_duplicate() {
    let fixture = clean_store("unaddressed-root");
    let mut bytes = fs::read(artifact_path(&fixture, Target::Root)).unwrap();
    bytes[28..36].copy_from_slice(&2_u64.to_le_bytes());
    bytes[48..56].copy_from_slice(&2_u64.to_le_bytes());
    refresh_crc32c(&mut bytes);
    let relative = "families/records/roots/root-0000000000000002.manifest";
    fs::write(fixture.store.join(relative), bytes).unwrap();
    let report = observe_store(&request(&fixture)).unwrap();
    let candidate = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.relative_path() == relative)
        .unwrap();
    assert_eq!(
        candidate.outcome(),
        &OfflineIntegrityOutcome::Unknown(OfflineUnknownPhysicalReason::RootNotAddressed)
    );
    assert!(candidate.duplicates().is_empty());
    assert_eq!(report.counters().duplicate_identities(), 0);
    assert_eq!(
        target_observation(&report, Target::Root).outcome(),
        &OfflineIntegrityOutcome::Intact
    );
}

#[test]
fn selector_issued_format_must_match_the_addressed_root() {
    let fixture = clean_store("root-format-scope");
    let path = artifact_path(&fixture, Target::Root);
    let mut bytes = fs::read(&path).unwrap();
    bytes[12..16].copy_from_slice(&32_768_u32.to_le_bytes());
    refresh_crc32c(&mut bytes);
    fs::write(path, bytes).unwrap();
    let report = observe_store(&request(&fixture)).unwrap();
    let OfflineIntegrityOutcome::Damaged(localization) =
        target_observation(&report, Target::Root).outcome()
    else {
        panic!("format-substituted addressed root must be damaged");
    };
    assert_eq!(
        localization.cause(),
        OfflinePhysicalDamageCause::ScopeMismatch
    );
    assert_eq!(
        localization.field(),
        Some(OfflinePhysicalFormatField::EmbeddedFormat)
    );
    assert_eq!(
        localization
            .damaged_range()
            .map(|range| (range.offset(), range.length())),
        Some((10, 10))
    );
}

#[test]
fn candidate_format_must_match_its_canonical_role_before_duplicate_classification() {
    let fixture = clean_store("candidate-format-scope");
    let mut bytes = current_selector_bytes();
    bytes[12..16].copy_from_slice(&32_768_u32.to_le_bytes());
    refresh_crc32c(&mut bytes);
    let relative = "families/records/root-current-0000000000000002.candidate";
    fs::write(fixture.store.join(relative), bytes).unwrap();
    let report = observe_store(&request(&fixture)).unwrap();
    let candidate = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.relative_path() == relative)
        .unwrap();
    let OfflineIntegrityOutcome::Damaged(localization) = candidate.outcome() else {
        panic!("foreign-format candidate must be scoped out");
    };
    assert_eq!(
        localization.cause(),
        OfflinePhysicalDamageCause::ScopeMismatch
    );
    assert_eq!(
        localization.field(),
        Some(OfflinePhysicalFormatField::EmbeddedFormat)
    );
    assert!(candidate.duplicates().is_empty());
    assert_eq!(report.counters().duplicate_identities(), 0);
}

#[test]
fn empty_manifest_cannot_smuggle_a_segment_root() {
    let fixture = clean_store("empty-segment-root");
    let path = artifact_path(&fixture, Target::Root);
    let mut bytes = fs::read(&path).unwrap();
    bytes[208] = 1;
    refresh_crc32c(&mut bytes);
    fs::write(path, bytes).unwrap();
    let report = observe_store(&request(&fixture)).unwrap();
    let OfflineIntegrityOutcome::Damaged(localization) =
        target_observation(&report, Target::Root).outcome()
    else {
        panic!("empty segment root must be damaged");
    };
    assert_eq!(localization.cause(), OfflinePhysicalDamageCause::Pointer);
    assert_eq!(
        localization
            .damaged_range()
            .map(|range| (range.offset(), range.length())),
        Some((208, 1))
    );
}

#[test]
fn checksum_valid_selector_link_must_match_the_other_slot() {
    let fixture = clean_store("selector-link-scope");
    let path = artifact_path(&fixture, Target::Current);
    let mut bytes = fs::read(&path).unwrap();
    bytes[73..81].copy_from_slice(&12_u64.to_le_bytes());
    bytes[81..89].copy_from_slice(&2_u64.to_le_bytes());
    refresh_crc32c(&mut bytes);
    fs::write(path, bytes).unwrap();
    let report = observe_store(&request(&fixture)).unwrap();
    let OfflineIntegrityOutcome::Damaged(localization) =
        target_observation(&report, Target::Current).outcome()
    else {
        panic!("mismatched selector link must be damaged");
    };
    assert_eq!(localization.cause(), OfflinePhysicalDamageCause::Pointer);
    assert_eq!(
        localization.field(),
        Some(OfflinePhysicalFormatField::LinkedSelector)
    );
    assert_eq!(
        localization
            .damaged_range()
            .map(|range| (range.offset(), range.length())),
        Some((73, 16))
    );
}

fn target_observation(
    report: &worth_store_offline_integrity_observer::OfflineIntegrityReport,
    target: Target,
) -> &OfflineArtifactObservation {
    let path = match target {
        Target::Current => "families/records/root-current.selector",
        Target::Previous => "families/records/root-previous.selector",
        Target::Root => "families/records/roots/root-0000000000000001.manifest",
    };
    report
        .artifacts()
        .iter()
        .find(|artifact| artifact.relative_path() == path)
        .unwrap()
}
