use super::support::*;

const CHILD_CONTROL: &str = "WORTH_STORE_S10_MATERIALIZATION_CONTROL";
const CHILD_TARGET: &str = "WORTH_STORE_S10_MATERIALIZATION_TARGET";
const BUFFER_BYTES: usize = 29;

#[test]
fn fresh_process_resumes_the_exact_durable_materialization_plan() {
    if let (Some(control_path), Some(target_path)) = (
        std::env::var_os(CHILD_CONTROL),
        std::env::var_os(CHILD_TARGET),
    ) {
        run_recovery_child(control_path.into(), target_path.into());
        return;
    }

    let scenario = BackupScenario::new("materialization-plan-recovery");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-materialization-plan-recovery").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("durable cut");
    let mut interrupted = admitted
        .materialize(&scenario.target, BUFFER_BYTES, &control)
        .expect("durable materialization plan");
    interrupted
        .advance_boundary()
        .expect("first durable copy boundary")
        .expect("nonempty backup has a boundary");
    drop(interrupted);
    drop(control);

    let status = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg(
            "phase_1_6_tests::materialization_control_recovery::fresh_process_resumes_the_exact_durable_materialization_plan",
        )
        .arg("--nocapture")
        .env(CHILD_CONTROL, &scenario.control)
        .env(CHILD_TARGET, &scenario.target)
        .status()
        .expect("fresh materialization recovery process");
    assert!(status.success());

    let reopened = scenario.control_store();
    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &reopened,
        ControlStoreGeneration::from_raw(4).expect("verified generation"),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
    let ControlStoreTrustPosture::Selected(selected) = reopened.inspect_generations(&fencing)
    else {
        panic!("completed child history must remain selected");
    };
    assert!(selected.active_backup_recovery_handles().is_empty());
    assert_eq!(
        selected
            .recover_backup_reachability_leases()
            .expect("replay verified lease release")
            .live_index_snapshot()
            .expect("replayed lease index")
            .active_leases(),
        0
    );
}

fn run_recovery_child(control_path: std::path::PathBuf, target_path: std::path::PathBuf) {
    let control = OperationalControlStore::open_with_certified_topology(
        OperationalControlLocation::new(control_path, failure_domain("control-media")),
        [ProtectedOperationalMediaLocation::backup_target(
            &target_path,
            failure_domain("target-media"),
        )],
    )
    .expect("child reopens control media");
    let authority = crate::backup::export::current_authority("store.physical.default_instance");
    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &control,
        ControlStoreGeneration::from_raw(2).expect("materialization-plan generation"),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
    let ControlStoreTrustPosture::Selected(selected) = control.inspect_generations(&fencing) else {
        panic!("child must select the durable materialization plan");
    };
    let leases = selected
        .recover_backup_reachability_leases()
        .expect("child reconstructs source lease");
    let recoverable = recover_online_backups(selected)
        .next()
        .expect("one recoverable backup");
    let operation_id = recoverable.operation_id().clone();
    let plan = recoverable
        .materialization_plan()
        .expect("control replay carries the durable materialization plan")
        .clone();
    assert_eq!(
        plan.target_parent(),
        std::fs::canonicalize(target_path).expect("canonical target")
    );
    assert_eq!(plan.buffer_bytes(), BUFFER_BYTES);

    let completion = recoverable
        .readmit(&authority, &backup_custody(&authority), 4 * 1024)
        .expect("readmit exact durable cut")
        .materialize(plan.target_parent(), plan.buffer_bytes(), &control)
        .expect("resume exact durable plan and staging session")
        .finish()
        .expect("complete resumed materialization");
    let (materialized, cut) = completion.into_parts();
    let structural = verify_materialized_backup(
        materialized,
        OfflineInspectionBudget::bounded(4 * 1024, u64::MAX).expect("verification budget"),
    )
    .expect("independent verification after fresh-process resume");
    record_independent_backup_verification(&operation_id, structural, cut, &control, &leases)
        .expect("durably record verification and release source lease");
}
