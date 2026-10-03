//! Releasing a lagging directory watermark while a newer publication survives:
//! the replacement binding drops only the invalid watermark, keeps the newer
//! latest hint, and leaves catalog reads stale until an ordinary rebuild.

use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    AdmittedBlobScope, AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    BlobCheckpointLimit, BlobIngestDeclaration, BlobIngestFailure, BlobReadLimits,
    BlobReadOpenFailure, LayoutRebuildLimits, PhysicalLayoutDenial,
    PhysicalLayoutMaintenanceFailure, PhysicalMutationDeadline, PhysicalRecordResidencyPolicy,
    PhysicalSpeculativeWorkKind as Speculation, PublishedBlobGeneration, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::IndexedThroughBlobPublication;
use worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration;
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;

use super::{
    admitted_blob_scope, assert_absent, assert_retired, certified_release_serving, checkpoint,
    fresh_process, read_exact, recover_custody, recover_serving, release, selected_marker,
    shared_format, world, Scope, CHUNK, SCOPE_NAME,
};

/// Below the 4 MiB catalog source-validation grant, so the newer publication
/// becomes durable while its derived catalog is denied.
const MAINTENANCE_LOW_BYTES: u64 = 1 << 20;

#[test]
fn released_lagging_watermark_keeps_the_newer_latest_publication() {
    std::thread::Builder::new()
        .name("lagging-watermark-release".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("recovery journey worker")
        .join()
        .expect("recovery journey worker did not panic");
}

fn run() {
    let format = shared_format();
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery_with_format_and_manifest_capacity(
            "c11-lagging-watermark-release",
            format,
            512,
        )
        .expect("genuine KiB64 recovery world");
    let scope = admitted_blob_scope(SCOPE_NAME);
    let indexed_payload = vec![0x41; CHUNK + 17];
    let indexed = world::publish_one(&world, &scope, &indexed_payload);
    let indexed_marker = selected_marker(world.serving());
    checkpoint(world.serving(), [0xf1; 32]);
    let producer_policy = world.serving().residency_observation().admitted_policy();
    let recovery_bytes = producer_policy
        .operation_bytes()
        .min(producer_policy.scope_bytes(Scope::Recovery));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    // Only the Maintenance scope differs: the newer publication is durable,
    // but its catalog validation is denied, so the watermark lags.
    let policy = low_maintenance_policy(format);
    let configuration =
        PhysicalRecoveryStaticConfiguration::for_record_format(format.declaration())
            .with_residency_policy(policy)
            .expect("policy admitted for the configured format");
    let seal = recover_custody(certified_release_serving::request_with_configuration(
        &root,
        recovery_bytes,
        configuration,
    ));
    let serving = certified_release_serving::admit_serving_with_seal_format_and_policy(
        &root, seal, format, policy,
    );
    let newer = publish_unindexed(&serving, format, &scope, &world::payload());
    let newer_marker = selected_marker(&serving);
    assert_ne!(newer_marker, indexed_marker);
    assert_stale(&serving, &scope, newer, newer_marker, Some(indexed_marker));
    // Checkpoint compaction also needs Maintenance, so C8 replays this
    // publication from the WAL tail.
    serving.close();

    let serving = recover_serving(&root, format, recovery_bytes);
    assert_stale(&serving, &scope, newer, newer_marker, Some(indexed_marker));
    let receipt = release(
        &serving,
        world::shared_placement(format),
        indexed,
        indexed_marker,
    );
    assert_retired(&receipt);
    assert!(receipt.dropped_records().contains(&indexed_marker.record()));
    assert!(!receipt.dropped_records().contains(&newer_marker.record()));
    // The replacement binding clears only the invalid watermark; the newer
    // latest hint survives, so reads stay stale instead of scanning.
    assert_eq!(selected_marker(&serving), newer_marker);
    assert_stale(&serving, &scope, newer, newer_marker, None);
    assert!(!serving.certification_selected_layout_record(indexed_marker.record()));

    let rebuilt = serving
        .layouts()
        .unwrap()
        .rebuild(
            DurableArtifactFamilyId::BlobCatalog,
            LayoutRebuildLimits::new(
                NonZeroU64::new(1024).unwrap(),
                NonZeroU64::new(1024).unwrap(),
            ),
            world::shared_placement(format),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .expect("ordinary rebuild from the selected authoritative closure");
    assert_eq!(rebuilt.source(), newer_marker);
    assert_absent(&serving, &scope, indexed.object().bytes());
    assert_eq!(read_exact(&serving, &scope, newer, &world::payload()), 0);
    checkpoint(&serving, [0xf3; 32]);
    serving.close();

    fresh_process::assert_reopens(
        &root,
        indexed.object().bytes(),
        newer.object().bytes(),
        recovery_bytes,
        fresh_process::ExpectedReopen::Surviving { reuse_reads: 0 },
    );
}

fn publish_unindexed(
    serving: &ServingPhysicalRuntime,
    format: AdmittedPhysicalRecordFormat,
    scope: &AdmittedBlobScope,
    payload: &[u8],
) -> PublishedBlobGeneration {
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        payload.len() as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(
            declaration,
            world::shared_placement(format),
            (CHUNK / 2) as u64,
            limits,
        )
        .expect("physical ingest admits before the maintenance denial");
    for piece in payload.chunks(CHUNK / 2) {
        ingest.push(piece).expect("physical chunk append");
    }
    match ingest.finish() {
        Err(BlobIngestFailure::PublishedIndexPending {
            published,
            cause: PhysicalLayoutMaintenanceFailure::FullAuthority(_),
        }) => published,
        other => panic!("catalog validation must deny after durable publication: {other:?}"),
    }
}

fn assert_stale(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    newer: PublishedBlobGeneration,
    latest: IndexedThroughBlobPublication,
    indexed: Option<IndexedThroughBlobPublication>,
) {
    let resolved = serving.blobs().unwrap().resolve_publication(
        newer.object().bytes(),
        newer.generation().sequence(),
        scope,
        BlobReadLimits::new(NonZeroU64::new(256).unwrap()),
    );
    assert!(
        matches!(
            &resolved,
            Err(BlobReadOpenFailure::Layout(PhysicalLayoutDenial::IndexStaleRequiresRebuild {
                latest: observed_latest,
                indexed: observed_indexed,
            })) if *observed_latest == latest && *observed_indexed == indexed
        ),
        "stale catalog must deny without an authoritative scan: {resolved:?}"
    );
}

fn low_maintenance_policy(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedPhysicalRecordResidencyPolicy {
    let canonical = AdmittedPhysicalRecordResidencyPolicy::canonical(format);
    let bytes = |value| NonZeroU64::new(value).unwrap();
    let count = |value| std::num::NonZeroU32::new(value).unwrap();
    let mut builder = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(canonical.total_bytes()))
        .resident_bytes(bytes(canonical.resident_bytes()))
        .metadata_bytes(bytes(canonical.metadata_bytes()))
        .frame_entries(count(canonical.frame_entries()))
        .pinned_frames(count(canonical.pinned_frames()))
        .pin_leases(count(canonical.pin_leases()))
        .dirty_frames(count(canonical.dirty_frames()))
        .dirty_replacement_bytes(bytes(canonical.dirty_replacement_bytes()))
        .operation_bytes(bytes(canonical.operation_bytes()))
        .progress_headroom_bytes(64 << 10);
    for scope in [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        let value = match scope {
            Scope::Maintenance => MAINTENANCE_LOW_BYTES,
            _ => canonical.scope_bytes(scope),
        };
        builder = builder.scope_bytes(scope, bytes(value));
    }
    for kind in [
        Speculation::ReadAhead,
        Speculation::Prefetch,
        Speculation::WriteBehind,
    ] {
        builder = builder.speculative_frames(kind, count(canonical.speculative_frames(kind)));
    }
    builder
        .admit(format)
        .into_result()
        .expect("only the Maintenance scope differs")
}
