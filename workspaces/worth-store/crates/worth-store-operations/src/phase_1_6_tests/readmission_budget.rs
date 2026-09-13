use super::support::*;

#[test]
fn zero_readmission_budget_is_typed_and_preserves_the_retry_handle() {
    let scenario = BackupScenario::new("zero-readmission-budget");
    let authority = scenario.authority();
    let control = scenario.control_store();
    OnlineBackupIntent::new(
        OperationalOperationId::new("zero-readmission-budget").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("durable source lease");
    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &control,
        ControlStoreGeneration::from_raw(1).expect("source lease generation"),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
    let ControlStoreTrustPosture::Selected(selected) = control.inspect_generations(&fencing) else {
        panic!("durable backup must be selected");
    };
    let recoverable = recover_online_backups(selected)
        .next()
        .expect("recoverable backup");

    let denial = recoverable
        .readmit(&authority, &backup_custody(&authority), 0)
        .expect_err("zero buffer must be a typed denial");
    assert!(matches!(
        denial.source(),
        OnlineBackupReadmissionFailure::InvalidObservationBudget
    ));
    let (retry, _) = denial.into_retry();
    retry
        .readmit(&authority, &backup_custody(&authority), 4 * 1024)
        .expect("valid retry budget readmits the same cut")
        .abandon("test closeout", &control, &scenario.leases)
        .expect("release source lease");
}

#[test]
fn recovery_rejects_custody_from_another_store_and_preserves_the_exact_retry() {
    let scenario = BackupScenario::new("foreign-readmission-custody");
    let authority = scenario.authority();
    let foreign = crate::backup::export::current_authority("s10-foreign-readmission-authority");
    let control = scenario.control_store();
    OnlineBackupIntent::new(
        OperationalOperationId::new("foreign-readmission-custody").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("durable source lease");
    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &control,
        ControlStoreGeneration::from_raw(1).expect("source lease generation"),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
    let ControlStoreTrustPosture::Selected(selected) = control.inspect_generations(&fencing) else {
        panic!("durable backup must be selected");
    };
    let recoverable = recover_online_backups(selected)
        .next()
        .expect("recoverable backup");

    let denial = recoverable
        .readmit(&authority, &backup_custody(&foreign), 4 * 1024)
        .expect_err("foreign custody cannot readmit this Store's cut");
    assert!(matches!(
        denial.source(),
        OnlineBackupReadmissionFailure::Cut(
            BackupCutReadmissionDenial::SecurityScopeAuthorityChanged
        )
    ));
    let (retry, _) = denial.into_retry();
    retry
        .readmit(&authority, &backup_custody(&authority), 4 * 1024)
        .expect("the exact recovery handle remains retryable")
        .abandon("test closeout", &control, &scenario.leases)
        .expect("release source lease");
}
