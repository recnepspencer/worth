use super::support::*;

const CHILD_COMPLETION_MARKER: &str = "WORTH_STORE_FRESH_PROCESS_VERIFICATION_COMPLETE";

#[test]
fn independent_verification_reopens_the_bundle_in_a_fresh_process() {
    const CHILD_ROOT: &str = "WORTH_STORE_S10_FRESH_PROCESS_BUNDLE";
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        let materialized = BackupBundleFormatAuthority::canonical()
            .admit_materialized(std::path::PathBuf::from(root))
            .expect("child reopens canonical bundle");
        verify_materialized_backup(
            materialized,
            OfflineInspectionBudget::bounded(4 * 1024, u64::MAX).expect("budget"),
        )
        .expect("fresh-process verification");
        println!("{CHILD_COMPLETION_MARKER}");
        return;
    }

    let scenario = BackupScenario::new("fresh-process");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-fresh-process").expect("operation id"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("cut");
    let completion = admitted
        .materialize(&scenario.target, 19, &control)
        .expect("session")
        .finish()
        .expect("materialize");
    let bundle_root = completion.bundle().root().to_path_buf();
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg("phase_1_6_tests::fresh_process_verification::independent_verification_reopens_the_bundle_in_a_fresh_process")
        .arg("--nocapture")
        .env(CHILD_ROOT, bundle_root)
        .output()
        .expect("fresh verifier process");
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(CHILD_COMPLETION_MARKER),
        "fresh verifier child did not report completion: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
