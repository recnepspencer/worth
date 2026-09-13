use super::support::*;

#[test]
fn cuts_on_both_sides_of_root_publication_remain_coherent_and_independently_protected() {
    let (older, newer) = BackupScenario::paired_across_root_publication("root-race");
    let authority = older.authority();
    let control_directory = tempfile::tempdir().expect("independent control directory");
    let control = OperationalControlStore::open_with_certified_topology(
        OperationalControlLocation::new(
            control_directory.path().join("operations.log"),
            failure_domain("root-race-control"),
        ),
        [
            ProtectedOperationalMediaLocation::source(
                &older.source,
                failure_domain("older-source"),
            ),
            ProtectedOperationalMediaLocation::backup_target(
                &older.target,
                failure_domain("older-target"),
            ),
            ProtectedOperationalMediaLocation::source(
                &newer.source,
                failure_domain("newer-source"),
            ),
            ProtectedOperationalMediaLocation::backup_target(
                &newer.target,
                failure_domain("newer-target"),
            ),
        ],
    )
    .expect("control media independent from both cuts");
    let leases = BackupReachabilityLeaseRegistry::for_store_runtime();
    let older_admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-before-root-publication").expect("operation"),
        older.coordinates(),
        older.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &leases)
    .expect("older root cut");
    let newer_admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-after-root-publication").expect("operation"),
        newer.coordinates(),
        newer.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &leases)
    .expect("newer root cut");

    assert_ne!(
        older_admitted.cut().identity(),
        newer_admitted.cut().identity()
    );
    assert_eq!(older_admitted.cut().coordinates().root_generation(), 1);
    assert_eq!(newer_admitted.cut().coordinates().root_generation(), 2);
    assert_eq!(
        leases
            .live_index_snapshot()
            .expect("two-cut lease index")
            .active_leases(),
        2
    );
    assert_cut_artifacts_block_reclaim(&older_admitted, &leases);
    assert_cut_artifacts_block_reclaim(&newer_admitted, &leases);

    let mut mixed_artifacts = older.references().to_vec();
    let older_root = mixed_artifacts
        .iter()
        .position(|artifact| artifact.family() == BackupArtifactFamily::RootManifest)
        .expect("older root");
    mixed_artifacts[older_root] = newer
        .references()
        .iter()
        .find(|artifact| artifact.family() == BackupArtifactFamily::RootManifest)
        .expect("newer root")
        .clone();
    let mixed_manifest = BackupCutManifest::canonical(mixed_artifacts).expect("shaped mixed cut");
    assert!(matches!(
        OnlineBackupIntent::new(
            OperationalOperationId::new("backup-mixed-root-publication").expect("operation"),
            older.coordinates(),
            mixed_manifest,
            backup_custody(&authority),
        )
        .admit_cut(&authority, &control, &leases),
        Err(OnlineBackupAdmissionDenial::Cut(
            BackupCutAdmissionDenial::RootGenerationMismatch
        ))
    ));

    let older_completion = older_admitted
        .materialize(&older.target, 41, &control)
        .expect("older materialization")
        .finish()
        .expect("older bundle");
    let newer_completion = newer_admitted
        .materialize(&newer.target, 41, &control)
        .expect("newer materialization")
        .finish()
        .expect("newer bundle");
    let (older_bundle, older_cut) = older_completion.into_parts();
    let (newer_bundle, newer_cut) = newer_completion.into_parts();
    let older_verified = verify_materialized_backup(older_bundle, verification_budget(&older))
        .expect("older cut independently verifies");
    let newer_verified = verify_materialized_backup(newer_bundle, verification_budget(&newer))
        .expect("newer cut independently verifies");
    assert_eq!(
        older_verified.materialized().manifest().root_generation(),
        older_cut.coordinates().root_generation()
    );
    assert_eq!(
        newer_verified.materialized().manifest().root_generation(),
        newer_cut.coordinates().root_generation()
    );
}

fn assert_cut_artifacts_block_reclaim(
    admitted: &crate::AdmittedOnlineBackup,
    leases: &BackupReachabilityLeaseRegistry,
) {
    for artifact in admitted.cut().manifest().artifacts() {
        let evidence =
            ExecutedReachabilityEvidence::for_certification_reference(artifact.reclaim_reference());
        let hazards = HazardLeaseTable::with_capacity(
            HazardLeaseTableCapacity::bounded_slots(1).expect("capacity"),
        )
        .live_index_snapshot();
        let proof = ReclaimEligibilityProof::admit(
            evidence,
            hazards,
            leases.live_index_snapshot().expect("live cut leases"),
        )
        .expect("reclaim proof");
        assert!(matches!(
            proof.try_reclaim(),
            Err(ReclaimDenial::BlockedByBackupCut { .. })
        ));
        assert_eq!(proof.counters().active_backup_leases(), 2);
    }
}

fn verification_budget(scenario: &BackupScenario) -> OfflineInspectionBudget {
    OfflineInspectionBudget::bounded(
        4 * 1024,
        scenario
            .total_bytes()
            .saturating_mul(2)
            .saturating_add(64 * 1024),
    )
    .expect("verification budget")
}
