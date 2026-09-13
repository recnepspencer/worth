use super::support::*;

#[test]
fn every_component_chunk_and_artifact_sync_boundary_resumes_the_full_owner_bundle() {
    const BUFFER_BYTES: usize = 37;
    let sizing = BackupScenario::new("materialization-boundary");
    let boundary_count = sizing
        .references()
        .iter()
        .map(|artifact| artifact.bytes().div_ceil(BUFFER_BYTES as u64) + 1)
        .sum::<u64>();
    drop(sizing);

    for cut in 0..=boundary_count {
        let scenario = BackupScenario::new("materialization-boundary");
        let authority = scenario.authority();
        let control = scenario.control_store();
        let operation =
            OperationalOperationId::new(format!("backup-boundary-{cut}")).expect("operation");
        let coordinates = scenario.coordinates();
        let manifest = scenario.cut_manifest();
        let admitted = OnlineBackupIntent::new(
            operation.clone(),
            coordinates.clone(),
            manifest.clone(),
            backup_custody(&authority),
        )
        .admit_cut(&authority, &control, &scenario.leases)
        .expect("admitted cut");
        let mut session = admitted
            .materialize(&scenario.target, BUFFER_BYTES, &control)
            .expect("materialization session");
        let mut durable_artifacts = 0_u64;
        for boundary in 0..cut {
            let progress = session
                .advance_boundary()
                .expect("materialization boundary")
                .unwrap_or_else(|| panic!("boundary {boundary} of {cut} must exist"));
            match progress {
                worth_store_physical_backend::PhysicalBackupMaterializationProgress::BytesCopied(
                    copied,
                ) => {
                    assert!(copied.bytes_copied() <= BUFFER_BYTES as u64);
                    assert!(copied.artifact_bytes_copied() <= copied.artifact_total_bytes());
                }
                worth_store_physical_backend::PhysicalBackupMaterializationProgress::ArtifactDurable(
                    durable,
                ) => {
                    assert_eq!(durable.artifact_index() as u64, durable_artifacts);
                    assert!(durable.artifact_bytes() > 0);
                    durable_artifacts += 1;
                }
            }
        }
        drop(session);

        let resumed =
            OnlineBackupIntent::new(operation, coordinates, manifest, backup_custody(&authority))
                .admit_cut(&authority, &control, &scenario.leases)
                .expect("idempotently reopen the durable cut")
                .materialize(&scenario.target, BUFFER_BYTES, &control)
                .expect("resume materialization")
                .finish()
                .expect("converged materialization");
        let (materialized, cut) = resumed.into_parts();
        let structural = verify_materialized_backup(
            materialized,
            OfflineInspectionBudget::bounded(
                4 * 1024,
                scenario
                    .total_bytes()
                    .saturating_mul(2)
                    .saturating_add(64 * 1024),
            )
            .expect("verification budget"),
        )
        .unwrap_or_else(|denial| panic!("cut {cut:?} failed independent verification: {denial:?}"));
        assert!(structural.report().defects().is_empty());
        assert_eq!(
            structural.materialized().manifest().cut_identity(),
            cut.identity()
        );
    }
}
