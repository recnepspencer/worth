use super::support::*;
use crate::BackupMaterializationAbandonmentRetry;

#[test]
fn cancelled_materialization_durably_releases_its_cut_and_removes_incomplete_output() {
    let scenario = BackupScenario::new("cancel-materialization");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-cancel-materialization").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("admitted cut");
    let cut_identity = admitted.cut().identity();
    let mut session = admitted
        .materialize(&scenario.target, 17, &control)
        .expect("materialization session");
    session.advance().expect("one copy boundary");
    let cancellation =
        worth_store_physical_backend::PhysicalBackupMaterializationCancellation::new();
    cancellation.cancel();
    assert!(matches!(
        session.advance_with_cancellation(&cancellation),
        Err(worth_store_physical_backend::PhysicalBackupMaterializationDenial::Cancelled)
    ));

    let abandoned = session
        .abandon("operator cancelled copy", &control, &scenario.leases)
        .expect("durable materialization abandonment");
    assert_eq!(abandoned.cut_receipt().cut_identity(), cut_identity);
    assert!(abandoned.cut_receipt().release_control_generation() > 0);
    let cleanup = abandoned
        .physical_cleanup()
        .expect("incomplete output cleanup");
    assert!(cleanup.incomplete_output_removed());
    assert!(!cleanup.incomplete_root().exists());
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("lease index")
            .active_leases(),
        0
    );
}

#[test]
fn abandonment_control_failure_preserves_the_live_session_for_an_exact_retry() {
    let scenario = BackupScenario::new("abandon-materialization-retry");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-abandon-materialization-retry").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("admitted cut");
    let mut session = admitted
        .materialize(&scenario.target, 13, &control)
        .expect("materialization session");
    session.advance().expect("partial copy");

    let denial = session
        .abandon(
            "retry durable cancellation",
            &FailingControlStore,
            &scenario.leases,
        )
        .expect_err("control failure must preserve retry state");
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("lease index")
            .active_leases(),
        1
    );
    let (retry, _) = denial.into_retry();
    let BackupMaterializationAbandonmentRetry::Materialization(session) = retry else {
        panic!("copy-phase retry must retain a materialization session");
    };
    let abandoned = session
        .abandon("retry durable cancellation", &control, &scenario.leases)
        .expect("retry abandonment on healthy control media");
    assert!(abandoned
        .physical_cleanup()
        .expect("cleanup")
        .incomplete_output_removed());
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("lease index")
            .active_leases(),
        0
    );
}

#[test]
fn cancellation_during_manifest_publication_abandons_without_publishing_a_backup() {
    let scenario = BackupScenario::new("cancel-publication");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-cancel-publication").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("admitted cut");
    let mut materialization = admitted
        .materialize(&scenario.target, 29, &control)
        .expect("materialization session");
    while materialization.advance().expect("copy") {}
    let mut publication = materialization
        .begin_publication()
        .expect("publication session");
    publication.advance().expect("pending manifest durability");
    let cancellation =
        worth_store_physical_backend::PhysicalBackupMaterializationCancellation::new();
    cancellation.cancel();
    assert!(matches!(
        publication.advance_with_cancellation(&cancellation),
        Err(worth_store_physical_backend::PhysicalBackupMaterializationDenial::Cancelled)
    ));
    let abandoned = publication
        .abandon("operator cancelled publication", &control, &scenario.leases)
        .expect("durable publication abandonment");
    let cleanup = abandoned.physical_cleanup().expect("staging cleanup");
    assert!(cleanup.incomplete_output_removed());
    assert!(!cleanup.completed_bundle_retained());
}

#[test]
fn every_owner_component_copy_and_sync_boundary_is_cancellable_without_advancing() {
    let scenario = BackupScenario::new("cancel-every-owner-boundary");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-cancel-every-owner-boundary").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("admitted cut");
    let mut session = admitted
        .materialize(&scenario.target, 31, &control)
        .expect("materialization session");
    let mut durable_artifacts = 0_usize;
    loop {
        let cancellation =
            worth_store_physical_backend::PhysicalBackupMaterializationCancellation::new();
        cancellation.cancel();
        let denied = session.advance_boundary_with_cancellation(&cancellation);
        if durable_artifacts == scenario.references().len() && matches!(denied, Ok(None)) {
            break;
        }
        assert!(matches!(
            denied,
            Err(worth_store_physical_backend::PhysicalBackupMaterializationDenial::Cancelled)
        ));
        let progress = session
            .advance_boundary()
            .expect("uncancelled boundary")
            .expect("remaining materialization boundary");
        if let worth_store_physical_backend::PhysicalBackupMaterializationProgress::ArtifactDurable(
            durable,
        ) = progress
        {
            assert_eq!(durable.artifact_index(), durable_artifacts);
            durable_artifacts += 1;
        }
    }
    assert_eq!(durable_artifacts, scenario.references().len());
    session
        .begin_publication()
        .expect("publication")
        .finish()
        .expect("completed backup after interruption matrix");
}
