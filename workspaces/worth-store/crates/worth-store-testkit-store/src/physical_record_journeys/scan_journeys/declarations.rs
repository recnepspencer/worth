pub use {crate::durable_publication::root::publish_single, crate::physical_record_journeys::media, crate::physical_record_journeys::scenario_configuration::dense_configuration, crate::physical_record_journeys::success};
pub use {worth_store::physical_runtime::ExternalRecordScanCursor, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordScanCounterSnapshot, worth_store::physical_runtime::RecordScanDenial, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest};

pub fn collect_scan(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    width: u32,
    scratch_bytes: usize,
) -> Vec<(worth_store::physical_runtime::PhysicalRecordId, Vec<u8>)> {
    collect_scan_evidence(serving, width, scratch_bytes).records
}

pub fn collect_scan_evidence(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    width: u32,
    scratch_bytes: usize,
) -> CollectedScan {
    let scan = serving
        .records()
        .expect("read protection admission")
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(width).unwrap()),
        )
        .unwrap();
    collect_session(scan, scratch_bytes)
}

pub fn collect_resume(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    cursor: ExternalRecordScanCursor,
    width: u32,
    scratch_bytes: usize,
) -> Vec<(worth_store::physical_runtime::PhysicalRecordId, Vec<u8>)> {
    let scan = serving
        .records()
        .expect("read protection admission")
        .scan(
            worth_store::physical_runtime::RecordScanRequest::resume(cursor)
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(width).unwrap()),
        )
        .unwrap();
    collect_session(scan, scratch_bytes).records
}

pub fn collect_session(
    mut scan: worth_store::physical_runtime::PhysicalRecordScanSession,
    scratch_bytes: usize,
) -> CollectedScan {
    let mut scratch = vec![0_u8; scratch_bytes];
    let mut found = Vec::new();
    let mut batches = Vec::new();
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for index in 0..batch.records().len() {
            found.push((
                batch.records()[index].record_id(),
                batch.payload(index).unwrap().to_vec(),
            ));
        }
        batches.push(batch.observation());
        if batch.is_complete() {
            break;
        }
    }
    CollectedScan {
        records: found,
        batches,
    }
}

pub struct CollectedScan {
    pub records: Vec<(worth_store::physical_runtime::PhysicalRecordId, Vec<u8>)>,
    pub batches: Vec<RecordScanCounterSnapshot>,
}

pub fn assert_exact_batch_accounting(
    width: u32,
    payloads: &[Vec<u8>],
    batches: &[RecordScanCounterSnapshot],
) {
    const DURABLE_FRAME_HEADER_BYTES: u64 = crate::durable_frame_oracle::HEADER_BYTES as u64;
    const ROOT_ROUTING_PREFIX_BYTES: u64 = 40;
    const ROOT_PLACEMENT_BYTES: u64 = 88;
    const MEMBERSHIP_BLOCK_BYTES: u64 = DURABLE_FRAME_HEADER_BYTES + 40 + 40;
    const EXTENT_MANIFEST_BYTES: u64 = DURABLE_FRAME_HEADER_BYTES + 56;

    let expected_batches = payloads.len().div_ceil(width as usize);
    assert_eq!(batches.len(), expected_batches);
    for (batch_index, observation) in batches.iter().copied().enumerate() {
        let records = ((batch_index + 1) * width as usize).min(payloads.len());
        let extents = (0..records).filter(|index| index % 4 == 0).count();
        let inline = records - extents;
        let root_bytes = DURABLE_FRAME_HEADER_BYTES
            + ROOT_ROUTING_PREFIX_BYTES
            + payloads.len() as u64 * ROOT_PLACEMENT_BYTES;
        assert_eq!(observation.records(), records as u64);
        assert_eq!(
            observation.payload_bytes(),
            payloads[..records]
                .iter()
                .map(|payload| payload.len() as u64)
                .sum::<u64>()
        );
        assert_eq!(observation.manifest_blocks(), 1 + records as u64);
        assert_eq!(observation.manifest_comparisons(), inline as u64);
        assert_eq!(
            observation.manifest_bytes(),
            root_bytes
                + inline as u64 * MEMBERSHIP_BLOCK_BYTES
                + extents as u64 * EXTENT_MANIFEST_BYTES
        );
        assert_eq!(observation.frames_traversed(), 1 + 2 * records as u64);
    }
}