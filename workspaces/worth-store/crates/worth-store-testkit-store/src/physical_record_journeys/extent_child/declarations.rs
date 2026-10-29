pub use std::io::Write;
pub use std::path::Path;

pub use {
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::PhysicalOperationAllocationScope,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};

pub use {crate::child_process::locator_codec::hex, crate::child_process::locator_codec::unhex};
pub use {
    crate::durable_publication, crate::physical_record_journeys::configuration,
    crate::physical_record_journeys::serving_from_initialization,
    crate::physical_record_journeys::serving_from_open,
};

pub fn allocation_writer(root: &Path, logical_bytes: &str) {
    let logical_bytes = logical_bytes.parse().unwrap();
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(root);
    let batch = worth_store::physical_runtime::RecordAppendBatch::builder()
        .push_source(crate::stream_fixture::PatternSource::exact(logical_bytes))
        .build()
        .unwrap();
    let published = crate::durable_publication::root::publish_single(
        &serving,
        placement,
        crate::durable_publication::material::certification_material(
            "cross-process-extent-allocation",
            1,
        ),
        batch,
    );
    let published_member = &published.settled_members()[0];
    println!(
        "C5_ALLOC {} {} {}",
        serving
            .residency_observation()
            .counters()
            .peak_operation_bytes(),
        published_member.observation().peak_scratch_bytes(),
        hex(
            &worth_store::physical_runtime::ExternalPhysicalRecordLocator::new(
                serving.store_identity(),
                published_member.record_id(0).unwrap(),
            )
            .encode()
        ),
    );
    std::io::stdout().flush().unwrap();
    serving.close();
}

pub fn allocation_reader(root: &Path, encoded_locator: &str) {
    let serving = serving_from_open(root);
    let locator = worth_store::physical_runtime::ExternalPhysicalRecordLocator::decode(unhex(
        encoded_locator,
    ))
    .unwrap();
    let observation = {
        let mut record = serving
            .records()
            .expect("read protection admission")
            .open_external(
                locator,
                worth_store::physical_runtime::RecordReadLimits::new(
                    worth_store::physical_runtime::RecordByteLimit::new(u32::MAX).unwrap(),
                ),
            )
            .unwrap();
        assert_eq!(record.read_next(&mut [0_u8; 1]).unwrap(), 1);
        record.observation()
    };
    println!(
        "C5_READ_ALLOC {} {}",
        serving
            .residency_observation()
            .counters()
            .peak_operation_bytes(),
        observation.peak_scratch_bytes(),
    );
    std::io::stdout().flush().unwrap();
    serving.close();
}

pub fn extent_reader(root: &Path, encoded_locator: &str) {
    let serving = serving_from_open(root);
    let locator = worth_store::physical_runtime::ExternalPhysicalRecordLocator::decode(unhex(
        encoded_locator,
    ))
    .unwrap();
    let mut record = serving
        .records()
        .expect("read protection admission")
        .open_external(
            locator,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(u32::MAX).unwrap(),
            ),
        )
        .unwrap();
    let widths = [
        1_usize, 2, 3, 7, 31, 127, 255, 511, 1024, 2047, 4093, 8191, 16_384, 97, 5, 333, 65_535,
    ];
    let mut total = 0_u64;
    let mut digest = 0_u64;
    let mut turn = 0;
    loop {
        let mut buffer = vec![0_u8; widths[turn % widths.len()]];
        let count = record.read_next(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        for byte in &buffer[..count] {
            digest = digest.rotate_left(5) ^ u64::from(*byte);
        }
        total += count as u64;
        turn += 1;
    }
    println!("C5_EXTENT {total} {digest}");
    let observation = record.observation();
    println!(
        "C5_EXTENT_OBS {} {} {} {} {} {} {}",
        observation.bytes_requested(),
        observation.bytes_completed(),
        observation.transfer_count(),
        observation.peak_transfer_width(),
        observation.copied_bytes(),
        observation.generation_checks(),
        observation.generation_rejections(),
    );
    std::io::stdout().flush().unwrap();
    serving.close();
}

pub fn scale_allocation_reader(root: &Path, encoded_locator: &str) {
    let format = crate::scale_support::format();
    let access = crate::scale_support::access(format, 7);
    let serving = crate::physical_record_journeys::success(open_record_store!(
        crate::physical_record_journeys::media(root),
        |durability| {
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
        }
    ));
    let locator = worth_store::physical_runtime::ExternalPhysicalRecordLocator::decode(unhex(
        encoded_locator,
    ))
    .unwrap();
    let record = serving
        .records()
        .expect("read protection admission")
        .readmit_locator(locator)
        .into_result()
        .unwrap();
    serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(100).unwrap(),
            ),
        )
        .unwrap()
        .observation();
    let point = serving
        .residency_observation()
        .counters()
        .peak_operation_bytes_for(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead,
        );
    crate::scale_support::complete_scan(&serving, 7, 16_384);
    let scan = serving
        .residency_observation()
        .counters()
        .peak_operation_bytes_for(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead,
        );
    // Fresh reopen admits a separate Recovery grant; its lifetime peak must
    // remain bounded, but it is not foreground point-read or scan scratch.
    let residency = serving.residency_observation();
    let counters = residency.counters();
    let policy = residency.admitted_policy();
    let recovery = worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery;
    assert!(counters.peak_operation_bytes() <= policy.operation_bytes());
    assert!(counters.peak_operation_bytes_for(recovery) > 0);
    assert!(counters.peak_operation_bytes_for(recovery) <= policy.scope_bytes(recovery));
    crate::scenario_process_evidence::emit_process("scale-allocation-probe", &serving);
    println!("C5_SCALE_ALLOC {point} {scan}");
    std::io::stdout().flush().unwrap();
    serving.close();
}
