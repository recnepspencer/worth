use super::release_scale_blob_fixture::write_sparse_zero_blob;
use super::support::*;

const TWO_GIB: u64 = 2 * 1024 * 1024 * 1024;
const STREAM_BUFFER_BYTES: usize = 128 * 1024;

#[test]
#[ignore = "scheduled release-scale Phase 6 lane"]
fn two_gib_owner_bundle_stays_bounded_through_materialization_and_independent_verification() {
    let mut scenario = BackupScenario::new("release-scale-backup");
    replace_blob_with_two_gib_owner_artifact(&mut scenario);
    let authority = scenario.authority();
    let control = scenario.control_store();
    let operation =
        OperationalOperationId::new("backup-release-scale").expect("operation identity");
    let admitted = OnlineBackupIntent::new(
        operation.clone(),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("owner-verified release-scale cut");
    let source_bytes = scenario.total_bytes();
    assert!(source_bytes > TWO_GIB);

    let completion = admitted
        .materialize(&scenario.target, STREAM_BUFFER_BYTES, &control)
        .expect("bounded release-scale materialization")
        .finish()
        .expect("durably publish release-scale bundle");
    let materialization = completion.counters();
    assert_eq!(materialization.source_bytes_read(), source_bytes);
    assert_eq!(materialization.output_bytes_written(), source_bytes);
    assert_eq!(
        materialization.peak_buffer_bytes(),
        STREAM_BUFFER_BYTES as u64
    );
    assert_eq!(
        materialization.logically_materialized_bytes(),
        Some(source_bytes)
    );

    let (materialized, cut) = completion.into_parts();
    let structural = verify_materialized_backup(
        materialized,
        OfflineInspectionBudget::bounded(STREAM_BUFFER_BYTES, u64::MAX)
            .expect("release verification budget"),
    )
    .expect("independent release-scale verification");
    assert!(structural.report().defects().is_empty());
    assert_eq!(
        structural.report().read_accounting(),
        BackupVerificationReadAccounting::Complete
    );
    assert_eq!(structural.report().owner_verified_bytes(), source_bytes);
    assert!(structural.report().peak_buffer_bytes() <= STREAM_BUFFER_BYTES as u64);
    assert!(structural.report().peak_owned_allocation_bytes() < 16 * 1024 * 1024);

    let verified = record_independent_backup_verification(
        &operation,
        structural,
        cut,
        &control,
        &scenario.leases,
    )
    .expect("durable verification and lease release");
    let qualified = qualify_backup_custody(verified, &backup_custody(&authority))
        .expect("release-scale custody qualification");
    admit_backup_for_production_restore(
        qualified,
        &authority,
        BackupRestoreAdmissionPolicy::production_default(),
    )
    .expect("release-scale production restore admission");
}

fn replace_blob_with_two_gib_owner_artifact(scenario: &mut BackupScenario) {
    let index = scenario
        .references
        .iter()
        .position(|artifact| artifact.family() == BackupArtifactFamily::BlobChunk)
        .expect("canonical bundle contains a blob");
    let original = scenario.references[index].clone();
    let path = scenario.source.join("release-scale-blob.media");
    let identity = write_sparse_zero_blob(&path, TWO_GIB).expect("sparse owner blob fixture");
    let observation = observe_physical_backup_artifact(&path, STREAM_BUFFER_BYTES)
        .expect("bounded physical observation of release-scale blob");
    assert_eq!(observation.peak_buffer_bytes(), STREAM_BUFFER_BYTES as u64);
    scenario.references[index] = BackupArtifactReference::declare_untrusted_physical_observation(
        UntrustedBackupArtifactClaim {
            family: BackupArtifactFamily::BlobChunk,
            format: BackupBundleArtifactFormat::BlobChunkV1,
            identity,
            generation: original.generation(),
            coverage: original.coverage().clone(),
        },
        observation,
        original.reclaim_reference(),
    )
    .expect("release-scale blob remains owner- and reclaim-bound");
}
