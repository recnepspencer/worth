use crate::phase_1_6_tests::support::{backup_custody, BackupScenario};
use crate::{
    admit_backup_for_production_restore, qualify_backup_custody,
    record_independent_backup_verification, OnlineBackupIntent, OperationalControlStore,
    OperationalOperationId, ProductionRestoreAdmissibleBackupBundle,
};
use worth_store_authority::BackupRestoreAdmissionPolicy;
use worth_store_offline_verifier::{verify_materialized_backup, OfflineInspectionBudget};

pub(crate) struct RestoreWorld {
    pub(crate) scenario: BackupScenario,
    pub(crate) authority: worth_store_authority::StoreCurrentAuthorityWitness,
    pub(crate) control: OperationalControlStore,
    pub(crate) admissible: ProductionRestoreAdmissibleBackupBundle,
    pub(crate) backup_root: std::path::PathBuf,
    pub(crate) restore_directory: tempfile::TempDir,
}

pub(crate) fn restore_world(case: &str) -> RestoreWorld {
    let scenario = BackupScenario::new(case);
    let authority = scenario.authority();
    let control = scenario.control_store();
    let operation_id = OperationalOperationId::new(format!("backup-{case}")).unwrap();
    let admitted = OnlineBackupIntent::new(
        operation_id.clone(),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("admit source cut");
    let completion = admitted
        .materialize(&scenario.target, 29, &control)
        .expect("open backup materialization")
        .finish()
        .expect("materialize backup");
    let (materialized, cut) = completion.into_parts();
    let structural = verify_materialized_backup(
        materialized,
        OfflineInspectionBudget::bounded(4 * 1024, u64::MAX).expect("verification budget"),
    )
    .expect("independent backup verification");
    let verified = record_independent_backup_verification(
        &operation_id,
        structural,
        cut,
        &control,
        &scenario.leases,
    )
    .expect("record independent verification");
    let qualified =
        qualify_backup_custody(verified, &backup_custody(&authority)).expect("qualify custody");
    let admissible = admit_backup_for_production_restore(
        qualified,
        &authority,
        BackupRestoreAdmissionPolicy::production_default(),
    )
    .expect("production restore admission");
    let backup_root = admissible
        .custody()
        .structural()
        .materialized()
        .root()
        .to_path_buf();
    RestoreWorld {
        scenario,
        authority,
        control,
        admissible,
        backup_root,
        restore_directory: tempfile::tempdir().expect("restore directory"),
    }
}
