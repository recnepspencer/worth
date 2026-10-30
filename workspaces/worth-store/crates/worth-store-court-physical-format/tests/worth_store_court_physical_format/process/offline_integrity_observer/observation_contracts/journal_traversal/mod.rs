use crate::{
    phase_4_literal_vectors::checkpoint_stream, phase_4_literal_vectors::pending_operation,
    phase_4_literal_vectors::wal_range, store_offline_integrity_observer_support::request,
    support::fixtures::clean_store,
};
use worth_store_offline_integrity_observer::observe_store;

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
        .filter(|row| row.outcome() == &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Unknown(worth_store_offline_integrity_observer::OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected))
        .collect();
    assert_eq!(aliases.len(), 14);
    for row in aliases {
        assert!(matches!(
            row.duplicates(),
            [worth_store_offline_integrity_observer::OfflineArtifactDuplicateEvidence::PhysicalAlias { .. }]
        ));
    }
    let output = std::process::Command::new(
        worth_store_testkit_executables::physical_store_integrity_observer(),
    )
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
        crate::store_offline_integrity_observer_support::FIXTURE_OPEN_FILE_ARGUMENT,
        "--max-depth",
        "8",
        "--max-symlinks",
        "0",
        "--max-elapsed-ms",
        crate::store_offline_integrity_observer_support::FIXTURE_ELAPSED_ARGUMENT,
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
