pub(crate) use {std::io::Write, std::path::Path};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PageFillPercent, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationDenial, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordAppendDenial, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::SegmentPageCount};
pub(crate) use {worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::allocation_probe::physical_record_journeys::allocations_during, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::durable_publication, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

pub(crate) fn batch() {
    let builder = (0..u16::MAX).fold(worth_store::physical_runtime::RecordAppendBatch::builder(), |builder, _| {
        builder.push_owned(Vec::new())
    });
    let payload = vec![0x5a; 1024 * 1024];
    let (builder, allocations) = allocations_during(|| builder.push_bytes(&payload));
    let denied = matches!(
        builder.build(),
        Err(worth_store::physical_runtime::RecordAppendDenial::BatchRecordLimitExceeded)
    );
    println!(
        "C5_BATCH_ADMISSION {} {} {denied}",
        allocations.allocations, allocations.bytes_allocated,
    );
    std::io::stdout().flush().unwrap();
}

pub(crate) fn geometry(root: &Path) {
    let (format, _, _) = configuration();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .segment_pages(worth_store::physical_runtime::SegmentPageCount::new(4).unwrap())
        .extent_threshold(worth_store::physical_runtime::RecordByteLimit::new(8_000).unwrap())
        .page_fill(worth_store::physical_runtime::PageFillPercent::new(1).unwrap())
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let serving = serving_from_initialization(root);
    let batch = worth_store::physical_runtime::RecordAppendBatch::builder()
        .push_source(worth_store_testkit_store::stream_fixture::PatternSource::exact(200))
        .build()
        .unwrap();
    let writes_before = serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    let (result, allocations) = allocations_during(|| {
        worth_store_testkit_store::durable_publication::wal_data::prepare_single(
            &serving.record_submission(),
            placement,
            worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([211; 32]),
            batch,
        )
        .into_raw()
    });
    let writes_after = serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    println!(
        "C5_GEOMETRY_ADMISSION {} {} {} {}",
        allocations.allocations,
        allocations.bytes_allocated,
        writes_after - writes_before,
        matches!(
            result,
            worth_proof::TransitionOutcome::Denied(worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
                worth_store::physical_runtime::RecordAppendDenial::InlinePageFull
            ))
        )
    );
    std::io::stdout().flush().unwrap();
    serving.close();
}