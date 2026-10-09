//! Destination-first release of a reused publication: the drop invalidates the
//! directory watermark, publishes a replacement binding in the same transition,
//! and the surviving source stays readable through ordinary catalog reads
//! before and after a checkpoint and fresh C8 reopen.

use worth_store::physical_runtime::{
    BlobReclaimDeferral, BlobReclaimDisposition, BlobReclaimFailure, ServingPhysicalRuntime,
};
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;

use super::{
    admitted_blob_scope, assert_absent, assert_retired, checkpoint, fresh_process, read_exact,
    recover_serving, release, release_request, selected_marker, shared_format, world, Scope,
    SCOPE_NAME,
};

#[test]
fn destination_first_release_keeps_the_source_routed_through_fresh_c8_custody() {
    std::thread::Builder::new()
        .name("destination-first-c8-custody".to_owned())
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
            "c11-shared-destination-first-c8",
            format,
            512,
        )
        .expect("genuine shared-reuse recovery world");
    let scope = admitted_blob_scope(SCOPE_NAME);
    let payload = world::payload();
    let original = world::publish_one(&world, &scope, &payload);
    let original_marker = selected_marker(world.serving());
    checkpoint(world.serving(), [0xe1; 32]);
    let destination = world::publish_one(&world, &scope, &payload);
    let destination_marker = selected_marker(world.serving());
    assert_eq!(
        read_exact(world.serving(), &scope, destination, &payload),
        6
    );

    // A held reader defers before any effect; the retry then completes exactly.
    let held = world.serving().records().unwrap();
    assert!(matches!(
        world.serving().blobs().unwrap().reclaim(release_request(
            world.serving(),
            world.placement(),
            destination,
            destination_marker,
        )),
        Err(BlobReclaimFailure::Deferred(
            BlobReclaimDeferral::ProtectedReader
        ))
    ));
    drop(held);
    assert_eq!(selected_marker(world.serving()), destination_marker);

    // The destination is the directory watermark, so this drop must publish
    // the replacement binding that retains the shared catalog root.
    let first = release(
        world.serving(),
        world.placement(),
        destination,
        destination_marker,
    );
    assert_retired(&first);
    assert_eq!(first.disposition(), BlobReclaimDisposition::Dropped);
    assert!(first
        .dropped_records()
        .contains(&destination_marker.record()));
    assert!(!first.dropped_records().contains(&original_marker.record()));
    assert_eq!(first.remaining_payload_records(), 0);
    assert_no_latest(world.serving());
    assert_absent(world.serving(), &scope, destination.object().bytes());
    assert_eq!(read_exact(world.serving(), &scope, original, &payload), 0);
    checkpoint(world.serving(), [0xe2; 32]);
    let producer_policy = world.serving().residency_observation().admitted_policy();
    let recovery_bytes = producer_policy
        .operation_bytes()
        .min(producer_policy.scope_bytes(Scope::Recovery));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    fresh_process::assert_reopens(
        &root,
        destination.object().bytes(),
        original.object().bytes(),
        recovery_bytes,
        fresh_process::ExpectedReopen::Surviving { reuse_reads: 0 },
    );
    let serving = recover_serving(&root, format, recovery_bytes);
    assert_no_latest(&serving);
    assert_absent(&serving, &scope, destination.object().bytes());
    assert_eq!(read_exact(&serving, &scope, original, &payload), 0);

    // The repeated destination release is proven to have no effect.
    let growth = serving.certification_charged_growth_bytes();
    let repeated = release(
        &serving,
        world::shared_placement(format),
        destination,
        destination_marker,
    );
    assert_eq!(
        repeated.disposition(),
        BlobReclaimDisposition::ProvenNoEffect
    );
    assert!(repeated.dropped_records().is_empty());
    assert_eq!(repeated.bytes_released(), 0);
    assert_eq!(serving.certification_charged_growth_bytes(), growth);

    let source = release(
        &serving,
        world::shared_placement(format),
        original,
        original_marker,
    );
    assert_retired(&source);
    assert!(source.dropped_records().contains(&original_marker.record()));
    assert_eq!(source.remaining_payload_records(), 0);
    assert_absent(&serving, &scope, original.object().bytes());
    checkpoint(&serving, [0xe3; 32]);
    serving.close();

    fresh_process::assert_reopens(
        &root,
        destination.object().bytes(),
        original.object().bytes(),
        recovery_bytes,
        fresh_process::ExpectedReopen::BothInvisible,
    );
}

fn assert_no_latest(serving: &ServingPhysicalRuntime) {
    assert_eq!(
        serving
            .certification_selected_latest_blob_publication()
            .unwrap(),
        None,
        "dropping the latest publication clears the selected latest hint",
    );
}
