//! Shared physical payload survives a source-first release and two fresh C8 seals.

use std::{
    num::{NonZeroU16, NonZeroU64},
    path::Path,
};

use worth_proof::{AdmittedBlobReleaseProof, TransitionOutcome};
use worth_store::physical_runtime::{
    AdmittedBlobScope, AdmittedPhysicalRecordFormat, BlobReadFailure, BlobReadLimits,
    BlobReadOpenFailure, BlobReclaimDisposition, BlobReclaimLimits, BlobReclaimReceipt,
    BlobReclaimRequest, BlobReclaimRetirement, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalMutationDeadline, PhysicalOperationAllocationScope as Scope, PhysicalPageSizeClass,
    PhysicalRecordFormatDeclaration, PublishedBlobGeneration, RecordReadDenial,
    ServingPhysicalRuntime,
};
use worth_store_physical_format::IndexedThroughBlobPublication;
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;

use super::super::{admitted_blob_scope, certified_release_serving};

#[path = "shared_reuse_custody/fresh_process.rs"]
mod fresh_process;
#[path = "shared_reuse_custody/world.rs"]
mod world;

const CHUNK: usize = 256 << 10;
const SCOPE_NAME: &str = "c11.recovery.release.shared.source.first";

#[test]
fn source_first_reuse_continues_only_with_fresh_c8_custody() {
    std::thread::Builder::new()
        .name("source-first-c8-custody".to_owned())
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
            "c11-shared-source-first-c8",
            format,
            512,
        )
        .expect("genuine shared-reuse recovery world");
    let scope = admitted_blob_scope(SCOPE_NAME);
    let payload = world::payload();
    let original = world::publish_one(&world, &scope, &payload);
    let original_marker = selected_marker(world.serving());
    checkpoint(world.serving(), [0xd1; 32]);
    let destination = world::publish_one(&world, &scope, &payload);
    let destination_marker = selected_marker(world.serving());
    read_exact(world.serving(), &scope, original, &payload);
    assert_eq!(
        read_exact(world.serving(), &scope, destination, &payload),
        4
    );
    for index in 0..4_u8 {
        world::publish_one(&world, &scope, &vec![0x20 + index; CHUNK]);
        checkpoint(world.serving(), [0xd2 + index; 32]);
    }

    let first = release(
        world.serving(),
        world.placement(),
        original,
        original_marker,
    );
    assert_retired(&first);
    assert!(first.dropped_records().contains(&original_marker.record()));
    assert!(first.remaining_payload_records() > 0);
    assert_absent(world.serving(), &scope, original.object().bytes());
    assert_eq!(
        read_exact(world.serving(), &scope, destination, &payload),
        4
    );
    checkpoint(world.serving(), [0xd6; 32]);
    let producer_policy = world.serving().residency_observation().admitted_policy();
    let recovery_bytes = producer_policy
        .operation_bytes()
        .min(producer_policy.scope_bytes(Scope::Recovery));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    fresh_process::assert_reopens(
        &root,
        original.object().bytes(),
        destination.object().bytes(),
        recovery_bytes,
        fresh_process::ExpectedReopen::SurvivingDestination,
    );
    let serving = recover_serving(&root, format, recovery_bytes);
    assert_absent(&serving, &scope, original.object().bytes());
    assert_eq!(read_exact(&serving, &scope, destination, &payload), 4);
    let released_destination = release(
        &serving,
        world::shared_placement(format),
        destination,
        destination_marker,
    );
    assert_retired(&released_destination);
    assert_eq!(released_destination.remaining_payload_records(), 0);
    assert_absent(&serving, &scope, destination.object().bytes());
    finish_source(
        &serving,
        world::shared_placement(format),
        original,
        original_marker,
    );
    checkpoint(&serving, [0xd7; 32]);
    serving.close();

    fresh_process::assert_reopens(
        &root,
        original.object().bytes(),
        destination.object().bytes(),
        recovery_bytes,
        fresh_process::ExpectedReopen::BothInvisible,
    );
}

#[test]
fn fresh_child_checks_source_first_reuse() {
    fresh_process::run_child();
}

fn shared_format() -> AdmittedPhysicalRecordFormat {
    AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .page_size(PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    )
}

fn recover_serving(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
    recovery_bytes: u64,
) -> ServingPhysicalRuntime {
    let outcome = WorthStoreRecovery::recover(certified_release_serving::request_with_memory(
        root,
        recovery_bytes,
    ));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            PhysicalRecoveryOutcome::Blocked(block) => panic!(
                "shared-reuse recovery blocked: kind={:?}; limit={:?}; effects={}",
                block.kind,
                block.evidence().limit,
                block.recovery_effects(),
            ),
            PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
                "shared-reuse recovery indeterminate: reopen={:?}; handoff={:?}; effects={}",
                failure.reopen_failure(),
                failure.handoff_failure(),
                failure.recovery_effects(),
            ),
            other => panic!("shared-reuse recovery did not produce custody: {other:?}"),
        }
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("C8-issued Store custody");
    certified_release_serving::admit_serving_with_seal_and_format(root, seal, format)
}

fn selected_marker(serving: &ServingPhysicalRuntime) -> IndexedThroughBlobPublication {
    serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("selected publication marker")
}

fn release(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    published: PublishedBlobGeneration,
    marker: IndexedThroughBlobPublication,
) -> BlobReclaimReceipt {
    serving
        .blobs()
        .unwrap()
        .reclaim(release_request(serving, placement, published, marker))
        .expect("selected release admitted")
        .wait()
        .expect("selected release completed")
}

fn release_request(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    published: PublishedBlobGeneration,
    marker: IndexedThroughBlobPublication,
) -> BlobReclaimRequest<'static> {
    let proof = AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        published.object().bytes(),
        published.generation().sequence(),
        marker.record().allocation_epoch(),
        marker.record().ordinal(),
        marker.encoded_digest(),
        [0x73; 32],
    )
    .unwrap();
    BlobReclaimRequest::released(
        proof,
        placement,
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(512).unwrap(),
            NonZeroU64::new(64 << 20).unwrap(),
            NonZeroU16::new(16).unwrap(),
        )
        .unwrap(),
    )
}

fn finish_source(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    published: PublishedBlobGeneration,
    marker: IndexedThroughBlobPublication,
) {
    for _ in 0..8 {
        let receipt = release(serving, placement, published, marker);
        assert_retired(&receipt);
        if receipt.remaining_payload_records() == 0 {
            let selected_growth = serving.certification_charged_growth_bytes();
            let repeated = serving
                .blobs()
                .unwrap()
                .reclaim(release_request(serving, placement, published, marker))
                .expect("terminal repeated release admitted")
                .wait()
                .expect("terminal repeated release completed");
            assert_eq!(
                repeated.disposition(),
                BlobReclaimDisposition::ProvenNoEffect
            );
            assert_eq!(repeated.bytes_released(), 0);
            assert_eq!(
                serving.certification_charged_growth_bytes(),
                selected_growth
            );
            return;
        }
    }
    panic!("source custody did not settle after the destination release");
}

fn assert_retired(receipt: &BlobReclaimReceipt) {
    assert_eq!(receipt.retirement(), BlobReclaimRetirement::Completed);
    assert!(receipt.bytes_released() > 0);
    assert_eq!(
        receipt.bytes_released(),
        receipt
            .displaced_extents()
            .iter()
            .map(|extent| extent.range().length())
            .sum::<u64>()
    );
}

fn checkpoint(serving: &ServingPhysicalRuntime, key: [u8; 32]) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("source-first checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}

fn assert_absent(serving: &ServingPhysicalRuntime, scope: &AdmittedBlobScope, object: [u8; 16]) {
    match serving.blobs().unwrap().resolve_publication(
        object,
        1,
        scope,
        BlobReadLimits::new(NonZeroU64::new(1).unwrap()),
    ) {
        Err(BlobReadOpenFailure::PublicationNotFound) => {}
        Err(BlobReadOpenFailure::Read(BlobReadFailure::RecordRead(error)))
            if error.denial() == RecordReadDenial::RecordNotFound => {}
        other => panic!("released source remained visible: {other:?}"),
    }
}

fn read_exact(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    published: PublishedBlobGeneration,
    expected: &[u8],
) -> u64 {
    let blobs = serving.blobs().unwrap();
    let mut read = blobs
        .read(
            published,
            scope,
            0,
            expected.len() as u64,
            BlobReadLimits::new(NonZeroU64::new(1).unwrap()),
        )
        .expect("selected destination read");
    let mut bytes = vec![0_u8; expected.len()];
    let mut used = 0;
    while used < bytes.len() {
        let count = read.read_next(&mut bytes[used..]).unwrap();
        assert!(count > 0);
        used += count;
    }
    assert_eq!(read.read_next(&mut bytes).unwrap(), 0);
    assert_eq!(bytes, expected);
    assert_eq!(read.observation().touched_chunks(), 2);
    read.observation().reuse_source_selected_reads()
}
