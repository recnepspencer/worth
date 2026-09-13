use super::support::*;

#[test]
fn backup_verification_honors_cancellation_before_reopening_bundle_media() {
    let scenario = BackupScenario::new("cancel-backup-verification");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let completion = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-cancel-verification").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("cut")
    .materialize(&scenario.target, 17, &control)
    .expect("session")
    .finish()
    .expect("materialize");
    let (materialized, _cut) = completion.into_parts();
    let cancellation = worth_store_offline_verifier::OfflineInspectionCancellation::new();
    cancellation.cancel();
    let denial = worth_store_offline_verifier::verify_materialized_backup_with_cancellation(
        materialized,
        OfflineInspectionBudget::bounded(4 * 1024, u64::MAX).expect("budget"),
        cancellation,
    )
    .expect_err("pre-cancelled verification cannot reopen component media");
    assert!(matches!(
        denial,
        BackupStructuralVerificationDenial::Inspection(OfflineInspectionDenial::Cancelled)
    ));
}
