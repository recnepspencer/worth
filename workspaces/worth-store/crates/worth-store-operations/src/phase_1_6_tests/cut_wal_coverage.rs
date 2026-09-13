use super::support::*;

#[test]
fn backup_cut_admission_rejects_a_wal_interval_not_closed_by_owner_coverage() {
    let scenario = BackupScenario::new("wal-gap");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let uncovered = BackupCutCoordinates::new(
        "lineage-a",
        1,
        1,
        scenario.checkpoint_identity(),
        10,
        10,
        13,
        13,
        "worth-physical-format-v1",
        "posix-file-fsync-dir-sync",
    )
    .expect("coordinate shape is individually valid");
    let denial = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-wal-gap").expect("operation"),
        uncovered,
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect_err("uncovered WAL frontier must fail");
    assert!(matches!(
        denial,
        OnlineBackupAdmissionDenial::Cut(BackupCutAdmissionDenial::WalCoverageGap)
    ));
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("lease registry")
            .active_leases(),
        0
    );
}
