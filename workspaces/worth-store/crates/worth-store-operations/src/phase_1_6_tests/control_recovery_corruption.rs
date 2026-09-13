use super::support::*;

#[test]
fn malformed_owner_lease_bytes_make_selected_control_state_damaged() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("control.log");
    let physical = worth_store_physical_backend::PhysicalOperationalControlStore::open(
        worth_store_physical_backend::ControlMediaLocation::new(&path),
    )
    .expect("physical control media");
    let malformed_object = physical
        .publish_recovery_object(b"not a backup cut recovery record")
        .expect("checksum-valid malformed recovery object");
    let scenario = BackupScenario::new("malformed-control-recovery");
    let authority = scenario.authority();
    let scenario_control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("recovery-source").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &scenario_control, &scenario.leases)
    .expect("admitted recovery source");
    let record_authority = admitted.cut().authority_identity();
    let recovery = admitted
        .cut()
        .recovery_record()
        .expect("valid recovery record");
    let operation = OperationalOperationId::new("damaged-lease").expect("operation");
    let transition = OperationalTransitionId::new("damaged-lease:source").expect("transition");
    let record = OperationalControlRecord::source_lease_persisted(
        record_authority,
        operation,
        transition,
        recovery,
        malformed_object,
    );
    let payload =
        crate::control_store::encode_control_record(&record).expect("binary control wire");
    physical
        .compare_exchange_append(None, "damaged-lease\0damaged-lease:source", &payload)
        .expect("checksum-valid semantic corruption");
    let control = OperationalControlStore::open(
        OperationalControlLocation::new(path, failure_domain("control-media")),
        std::iter::empty::<ProtectedOperationalMediaLocation>(),
    )
    .expect("operations control store");
    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &control,
        ControlStoreGeneration::initial(),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
    assert!(matches!(
        control.inspect_generations(&fencing),
        ControlStoreTrustPosture::Damaged(_)
    ));
}
