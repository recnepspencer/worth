use super::support::*;

#[test]
fn control_media_inside_protected_source_is_rejected() {
    let directory = tempfile::tempdir().expect("temp directory");
    let source = directory.path().join("source");
    std::fs::create_dir_all(&source).expect("source");
    let domain = failure_domain("same-media");
    assert!(OperationalControlStore::open(
        OperationalControlLocation::new(source.join("control.log"), domain.clone()),
        [ProtectedOperationalMediaLocation::source(&source, domain)],
    )
    .is_err());
}

#[test]
fn control_recovery_objects_cannot_overlap_protected_media() {
    let directory = tempfile::tempdir().expect("temp directory");
    let control = directory.path().join("control.log");
    let protected = directory.path().join("control.log.objects");

    assert!(OperationalControlStore::open(
        OperationalControlLocation::new(control, failure_domain("control-media")),
        [ProtectedOperationalMediaLocation::source(
            protected,
            failure_domain("separately-labelled-source-media"),
        )],
    )
    .is_err());
}

#[test]
fn distinct_configuration_labels_cannot_disguise_one_filesystem_failure_domain() {
    let directory = tempfile::tempdir().expect("temp directory");
    let control = directory.path().join("control").join("operations.log");
    let protected = directory.path().join("protected");
    std::fs::create_dir_all(control.parent().expect("control parent")).expect("control parent");
    std::fs::create_dir_all(&protected).expect("protected media");

    let denial = OperationalControlStore::open(
        OperationalControlLocation::new(control, failure_domain("label-control")),
        [ProtectedOperationalMediaLocation::source(
            protected,
            failure_domain("label-source"),
        )],
    )
    .expect_err("one filesystem is one observed failure domain");

    assert!(matches!(
        denial,
        OperationalControlStoreOpenDenial::SharedObservedFilesystem { .. }
    ));
}

#[test]
fn materialization_target_cannot_overlap_any_control_media_surface() {
    let scenario = BackupScenario::new("control-target-overlap");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-control-target-overlap").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("durable cut");
    let control_parent = scenario.control.parent().expect("control parent");

    let denial = match admitted.materialize(control_parent, 4096, &control) {
        Ok(_) => panic!("control media cannot also be backup staging media"),
        Err(denial) => denial,
    };
    assert!(matches!(
        denial,
        BackupMaterializationDenial::PlanPersistence(
            OperationalControlAppendDenial::ControlMediaOverlap { .. }
        )
    ));
    assert!(std::fs::read_dir(control_parent)
        .expect("control namespace")
        .all(|entry| !entry
            .expect("control entry")
            .file_name()
            .to_string_lossy()
            .contains("worth-backup")));
}

#[test]
fn source_media_cannot_be_reclassified_as_a_backup_target_after_control_open() {
    let scenario = BackupScenario::new("source-target-reclassification");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-source-target-reclassification").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("durable cut");

    let denial = match admitted.materialize(&scenario.source, 4096, &control) {
        Ok(_) => panic!("source media cannot become backup staging media"),
        Err(denial) => denial,
    };
    assert!(matches!(
        denial,
        BackupMaterializationDenial::PlanPersistence(
            OperationalControlAppendDenial::UnconfiguredMaterializationTarget { .. }
        )
    ));
    assert!(std::fs::read_dir(&scenario.source)
        .expect("source namespace")
        .all(|entry| !entry
            .expect("source entry")
            .file_name()
            .to_string_lossy()
            .contains("worth-backup")));
}
