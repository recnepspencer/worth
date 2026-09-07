use crate::{
    phase_4_literal_vectors::{checkpoint_stream, pending_operation, wal_range},
    support::{clean_store, request},
};
use worth_foundational::PhysicalArtifactFamily as Family;
use worth_store_offline_integrity_observer::{
    observe_store, OfflineArtifactDuplicateEvidence, OfflineIntegrityOutcome as Outcome,
    OfflinePhysicalDamageCause as Cause, OfflineUnknownPhysicalReason as Unknown,
};

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
    assert_eq!(row.family(), Family::RootRoutingBlock);
    assert_eq!(
        row.outcome(),
        &Outcome::Unknown(Unknown::ParentScopeUnavailable)
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
        matches!(row.outcome(), Outcome::Damaged(localization) if localization.cause() == Cause::ChecksumMismatch)
    );
}

#[test]
fn canonical_journal_alias_storm_does_not_repeat_reads_checksums_or_stream_vectors() {
    let fixture = clean_store("journal-alias-storm");
    let checkpoint = fixture.store.join("families/checkpoint.current");
    let staging = fixture.store.join("staging");
    let wal = fixture.store.join("families/wal");
    let pending = fixture.store.join("families/physical-work");
    for directory in [&staging, &wal, &pending] {
        std::fs::create_dir(directory).unwrap();
    }
    std::fs::write(&checkpoint, checkpoint_stream()).unwrap();
    let wal_path = wal.join("segment-1-generation-2.wal");
    std::fs::write(&wal_path, wal_range(1, 3, 4)).unwrap();
    let pending_path =
        pending.join("effect-0000000000000001-0000000000000002-0000000000000003.pending");
    std::fs::write(&pending_path, pending_operation()).unwrap();
    let before = observe_store(&request(&fixture)).unwrap();
    for sequence in 7..19 {
        std::fs::hard_link(
            &checkpoint,
            staging.join(format!("checkpoint-{sequence:016x}.candidate")),
        )
        .unwrap();
    }
    std::fs::hard_link(&wal_path, wal.join("segment-2-generation-2.wal")).unwrap();
    std::fs::hard_link(
        &pending_path,
        pending.join("effect-0000000000000001-0000000000000002-0000000000000004.pending"),
    )
    .unwrap();
    let after = observe_store(&request(&fixture)).unwrap();
    assert_eq!(
        after.counters().bytes_read(),
        before.counters().bytes_read()
    );
    assert_eq!(
        after.counters().checksum_calculations(),
        before.counters().checksum_calculations()
    );
    assert_eq!(after.artifacts().len(), before.artifacts().len() + 14);
    let aliases: Vec<_> = after
        .artifacts()
        .iter()
        .filter(|row| row.outcome() == &Outcome::Unknown(Unknown::PhysicalAliasNotReinspected))
        .collect();
    assert_eq!(aliases.len(), 14);
    for row in aliases {
        assert!(matches!(
            row.duplicates(),
            [OfflineArtifactDuplicateEvidence::PhysicalAlias { .. }]
        ));
    }
    let output =
        std::process::Command::new(env!("CARGO_BIN_EXE_physical_store_integrity_observer"))
            .arg("observe")
            .arg("--store-root")
            .arg(&fixture.store)
            .arg("--report")
            .arg("-")
            .args([
                "--max-entries",
                "100",
                "--max-bytes",
                "16384",
                "--max-open-files",
                "5",
                "--max-depth",
                "8",
                "--max-symlinks",
                "0",
                "--max-elapsed-ms",
                "10000",
                "--max-report-bytes",
                "65536",
            ])
            .output()
            .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let wire: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        wire["consumed"]["checksum_calculations"],
        after.counters().checksum_calculations()
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
    assert!(clean
        .artifacts()
        .iter()
        .any(|row| row.family() == Family::WalFrame && row.outcome() == &Outcome::Intact));
    std::fs::write(&path, []).unwrap();
    let damaged = observe_store(&request(&fixture)).unwrap();
    let rows: Vec<_> = damaged
        .artifacts()
        .iter()
        .filter(|row| row.relative_path().ends_with(".wal"))
        .collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].family(), Family::WalFrame);
    assert!(
        matches!(rows[0].outcome(), Outcome::Damaged(localization) if localization.cause() == Cause::Truncation)
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
        matches!(gaps[0].outcome(), Outcome::Damaged(localization) if localization.cause() == Cause::MissingArtifact)
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
