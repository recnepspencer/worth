//! Shared physical payload survives a source-first release and two fresh C8 seals.

pub(crate) use {std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path};

pub(crate) use worth_store_physical_format::IndexedThroughBlobPublication;
pub(crate) use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;
pub(crate) use {worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome};
pub(crate) use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::BlobReadFailure, worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReadOpenFailure,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimReceipt,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::BlobReclaimRetirement,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope,
    worth_store::physical_runtime::PhysicalPageSizeClass,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PublishedBlobGeneration,
    worth_store::physical_runtime::RecordReadDenial,
    worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub(crate) use {
    worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
};

pub(crate) const CHUNK: usize = 256 << 10;
pub(crate) const SCOPE_NAME: &str = "c11.recovery.release.shared.source.first";

pub(crate) fn run() {
    let format = shared_format();
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_operation_budget(
        "c11-shared-source-first-c8",
        format,
        512,
        crate::production_entry::release_reopen::shared_reuse_custody::world::release_operation_budget(),
    )
    .expect("genuine shared-reuse recovery world");
    let scope = admitted_blob_scope(SCOPE_NAME);
    let payload = crate::production_entry::release_reopen::shared_reuse_custody::world::payload();
    let original =
        crate::production_entry::release_reopen::shared_reuse_custody::world::publish_one(
            &world, &scope, &payload,
        );
    let original_marker = selected_marker(world.serving());
    checkpoint(world.serving(), [0xd1; 32]);
    let destination =
        crate::production_entry::release_reopen::shared_reuse_custody::world::publish_one(
            &world, &scope, &payload,
        );
    let destination_marker = selected_marker(world.serving());
    read_exact(world.serving(), &scope, original, &payload);
    // Both reuse claims still select chunk, publication, and leaf witnesses.
    assert_eq!(
        read_exact(world.serving(), &scope, destination, &payload),
        6
    );
    for index in 0..4_u8 {
        crate::production_entry::release_reopen::shared_reuse_custody::world::publish_one(
            &world,
            &scope,
            &vec![0x20 + index; CHUNK],
        );
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
        .min(producer_policy.scope_bytes(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
        ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    crate::production_entry::release_reopen::shared_reuse_custody::fresh_process::assert_reopens(
        &root,
        original.object().bytes(),
        destination.object().bytes(),
        recovery_bytes,
        crate::production_entry::release_reopen::shared_reuse_custody::fresh_process::ExpectedReopen::Surviving { reuse_reads: 4 },
    );
    let serving = recover_serving(&root, format, recovery_bytes);
    assert_absent(&serving, &scope, original.object().bytes());
    assert_eq!(read_exact(&serving, &scope, destination, &payload), 4);
    let released_destination = release(
        &serving,
        crate::production_entry::release_reopen::shared_reuse_custody::world::shared_placement(
            format,
        ),
        destination,
        destination_marker,
    );
    assert_retired(&released_destination);
    assert_eq!(released_destination.remaining_payload_records(), 0);
    assert_absent(&serving, &scope, destination.object().bytes());
    finish_source(
        &serving,
        crate::production_entry::release_reopen::shared_reuse_custody::world::shared_placement(
            format,
        ),
        original,
        original_marker,
    );
    checkpoint(&serving, [0xd7; 32]);
    serving.close();

    crate::production_entry::release_reopen::shared_reuse_custody::fresh_process::assert_reopens(
        &root,
        original.object().bytes(),
        destination.object().bytes(),
        recovery_bytes,
        crate::production_entry::release_reopen::shared_reuse_custody::fresh_process::ExpectedReopen::BothInvisible,
    );
}

pub(crate) fn shared_format() -> AdmittedPhysicalRecordFormat {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .page_size(worth_store::physical_runtime::PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    )
}

pub(crate) fn recover_serving(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
    recovery_bytes: u64,
) -> ServingPhysicalRuntime {
    let seal = recover_custody(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_memory_and_format(
        root,
        recovery_bytes,
        format,
    ));
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal_and_format(root, seal, format)
}

pub(crate) fn recover_custody(
    request: PhysicalRecoveryOpenRequest,
) -> RecoveredPhysicalCheckpointCustody {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(request);
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(block) => panic!(
                "shared-reuse recovery blocked: kind={:?}; artifact={:?}; generation={:?}; source_denials={:?}; discovery={:?}; integrity={:?}; limit={:?}; effects={}",
                block.cause(),
                block.evidence().artifact.as_deref(),
                block.evidence().source_generation,
                block.evidence().source_denials.as_slice(),
                block.evidence().counters,
                block.evidence().integrity_counters(),
                block.cause().limit(),
                block.recovery_effects(),
            ),
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
                "shared-reuse recovery indeterminate: reopen={:?}; handoff={:?}; effects={}",
                failure.reopen_failure(),
                failure.handoff_failure(),
                failure.recovery_effects(),
            ),
            other => panic!("shared-reuse recovery did not produce custody: {other:?}"),
        }
    };
    handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("C8-issued Store custody")
}

pub(crate) fn selected_marker(serving: &ServingPhysicalRuntime) -> IndexedThroughBlobPublication {
    serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("selected publication marker")
}

pub(crate) fn release(
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

pub(crate) fn release_request(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    published: PublishedBlobGeneration,
    marker: IndexedThroughBlobPublication,
) -> BlobReclaimRequest<'static> {
    let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        published.object().bytes(),
        published.generation().sequence(),
        marker.record().allocation_epoch(),
        marker.record().ordinal(),
        marker.encoded_digest(),
        [0x73; 32],
    )
    .unwrap();
    worth_store::physical_runtime::BlobReclaimRequest::released(
        proof,
        placement,
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(512).unwrap(),
            std::num::NonZeroU64::new(64 << 20).unwrap(),
            std::num::NonZeroU16::new(16).unwrap(),
        )
        .unwrap(),
    )
}

pub(crate) fn finish_source(
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
                worth_store::physical_runtime::BlobReclaimDisposition::ProvenNoEffect
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

pub(crate) fn assert_retired(receipt: &BlobReclaimReceipt) {
    assert_eq!(
        receipt.retirement(),
        worth_store::physical_runtime::BlobReclaimRetirement::Completed
    );
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

pub(crate) fn checkpoint(serving: &ServingPhysicalRuntime, key: [u8; 32]) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("source-first checkpoint must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(
            outcome,
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
        "checkpoint {key:02x?}: {outcome:?}"
    );
}

pub(crate) fn assert_absent(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
    object: [u8; 16],
) {
    match serving.blobs().unwrap().resolve_publication(
        object,
        1,
        scope,
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(1).unwrap()),
    ) {
        Err(worth_store::physical_runtime::BlobReadOpenFailure::PublicationNotFound) => {}
        Err(worth_store::physical_runtime::BlobReadOpenFailure::Read(
            worth_store::physical_runtime::BlobReadFailure::RecordRead(error),
        )) if error.denial() == worth_store::physical_runtime::RecordReadDenial::RecordNotFound => {
        }
        other => panic!("released source remained visible: {other:?}"),
    }
}

pub(crate) fn read_exact(
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
            worth_store::physical_runtime::BlobReadLimits::new(
                std::num::NonZeroU64::new(1).unwrap(),
            ),
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
