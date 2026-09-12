use worth_store::physical_runtime::{
    ExternalRecordScanCursor, PhysicalRecordAccessPolicy, PhysicalRecordInitialization,
    RecordAppendBatch, RecordByteLimit, RecordCountLimit, RecordReadLimits,
    RecordScanCounterSnapshot, RecordScanDenial, RecordScanOutcome, RecordScanRequest,
};

use super::{
    durable_publication::publish_single, media, scenario_configuration::dense_configuration,
    success,
};

#[test]
fn scan_batch_widths_converge_to_one_physical_sequence() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(2);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let payloads = (0_u8..13)
        .map(|ordinal| {
            let length = if ordinal % 4 == 0 {
                9_000
            } else {
                400 + usize::from(ordinal)
            };
            vec![ordinal; length]
        })
        .collect::<Vec<_>>();
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([160; 32]),
        RecordAppendBatch::try_from_iter(payloads.iter()).unwrap(),
    );
    for width in [1, 3, 7, 13] {
        let evidence = collect_scan_evidence(&serving, width, 64_000);
        assert_eq!(evidence.records.len(), payloads.len());
        for (index, (record, bytes)) in evidence.records.into_iter().enumerate() {
            assert_eq!(
                record,
                published.settled_members()[0].record_id(index).unwrap()
            );
            assert_eq!(bytes, payloads[index]);
        }
        assert_exact_batch_accounting(width, &payloads, &evidence.batches);
    }

    let mut first = serving
        .records()
        .expect("read protection admission")
        .scan(RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(3).unwrap()))
        .unwrap();
    let mut scratch = vec![0_u8; 32_000];
    let cursor = match first.read_next_into(&mut scratch).unwrap() {
        RecordScanOutcome::Batch(batch) => {
            assert_eq!(batch.records().len(), 3);
            batch.end_cursor()
        }
        RecordScanOutcome::Completed(_) => panic!("the seeded scan cannot complete empty"),
    };
    drop(first);
    let resumed = collect_resume(&serving, cursor, 4, 32_000);
    assert_eq!(resumed.len(), payloads.len() - 3);
    assert_eq!(
        resumed[0].0,
        published.settled_members()[0].record_id(3).unwrap()
    );
    serving.close();
}

#[test]
fn whole_extent_materialization_mutant_is_replaced_by_deferred_scan_payload() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, _) = dense_configuration(4);
    let access = PhysicalRecordAccessPolicy::builder()
        .transfer_limit(RecordByteLimit::new(65_536).unwrap())
        .scratch_limit(RecordByteLimit::new(131_072).unwrap())
        .admit(format)
        .unwrap();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let logical_bytes = 17 * 65_536 + 7;
    let batch = RecordAppendBatch::builder()
        .push_source(super::stream_fixture::PatternSource::exact(logical_bytes))
        .build()
        .unwrap();
    let publication = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([161; 32]),
        batch,
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();

    let mut scan = serving
        .records()
        .expect("read protection admission")
        .scan(RecordScanRequest::from_start())
        .unwrap();
    let mut scratch = vec![0_u8; 131_072];
    let batch = match scan.read_next_into(&mut scratch).unwrap() {
        RecordScanOutcome::Batch(batch) => batch,
        RecordScanOutcome::Completed(_) => panic!("the extent record must be discoverable"),
    };
    assert_eq!(batch.records().len(), 1);
    assert_eq!(batch.records()[0].record_id(), record);
    assert_eq!(batch.records()[0].declared_payload_bytes(), logical_bytes);
    assert!(batch.records()[0].payload_is_deferred());
    assert!(batch.payload(0).is_none());
    drop(batch);

    let mut read = serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(logical_bytes as u32).unwrap()),
        )
        .unwrap();
    let mut transferred = 0_u64;
    let mut width = vec![0_u8; 65_536];
    loop {
        let count = read.read_next(&mut width).unwrap();
        if count == 0 {
            break;
        }
        transferred += count as u64;
    }
    assert_eq!(transferred, logical_bytes);
    assert!(read.observation().peak_transfer_width() <= 65_536);
    serving.close();
}

#[test]
fn stale_foreign_and_out_of_range_cursors_fail_before_payload_read() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(2);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([162; 32]),
        RecordAppendBatch::try_from_iter([b"alpha".as_slice(), b"beta".as_slice()]).unwrap(),
    );
    let mut scan = serving
        .records()
        .expect("read protection admission")
        .scan(RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(1).unwrap()))
        .unwrap();
    let mut scratch = [0_u8; 16];
    let valid = match scan.read_next_into(&mut scratch).unwrap() {
        RecordScanOutcome::Batch(batch) => batch.end_cursor(),
        RecordScanOutcome::Completed(_) => unreachable!(),
    };
    drop(scan);

    for (offset, expected) in [
        (0, RecordScanDenial::ForeignStore),
        (16, RecordScanDenial::StaleRoot),
        (24, RecordScanDenial::RoutingTreeMismatch),
        (32, RecordScanDenial::FormatMismatch),
        (42, RecordScanDenial::CursorPositionNotFound),
    ] {
        let mut bytes = valid.encode();
        bytes[offset] ^= 0x7f;
        let forged = ExternalRecordScanCursor::decode(bytes).unwrap();
        let error = match serving
            .records()
            .expect("read protection admission")
            .scan(RecordScanRequest::resume(forged))
        {
            Ok(_) => panic!("forged scan cursor was admitted"),
            Err(error) => error,
        };
        assert_eq!(error.denial(), expected);
        assert_eq!(error.observation().payload_bytes(), 0);
        assert_eq!(error.observation().records(), 0);
    }
    serving.close();
}

#[test]
fn scratch_retry_preserves_position_and_counts_manifest_discovery_once() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(2);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([163; 32]),
        RecordAppendBatch::try_from_iter([b"alpha".as_slice(), b"beta".as_slice()]).unwrap(),
    );
    let mut scan = serving
        .records()
        .expect("read protection admission")
        .scan(RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(1).unwrap()))
        .unwrap();
    let error = match scan.read_next_into(&mut []) {
        Ok(_) => panic!("empty scratch cannot hold the first record"),
        Err(error) => error,
    };
    assert_eq!(
        error.denial(),
        RecordScanDenial::CallerScratchTooSmall { required: 5 }
    );
    assert!(error.observation().manifest_blocks() > 0);
    let discovered = error.observation().manifest_blocks();
    let mut scratch = [0_u8; 5];
    let batch = match scan.read_next_into(&mut scratch).unwrap() {
        RecordScanOutcome::Batch(batch) => batch,
        RecordScanOutcome::Completed(_) => unreachable!(),
    };
    assert_eq!(batch.payload(0), Some(b"alpha".as_slice()));
    assert_eq!(batch.observation().manifest_blocks(), discovered + 1);
    drop(scan);
    serving.close();
}

pub(super) fn collect_scan(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    width: u32,
    scratch_bytes: usize,
) -> Vec<(worth_store::physical_runtime::PhysicalRecordId, Vec<u8>)> {
    collect_scan_evidence(serving, width, scratch_bytes).records
}

fn collect_scan_evidence(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    width: u32,
    scratch_bytes: usize,
) -> CollectedScan {
    let scan = serving
        .records()
        .expect("read protection admission")
        .scan(
            RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(width).unwrap()),
        )
        .unwrap();
    collect_session(scan, scratch_bytes)
}

fn collect_resume(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    cursor: ExternalRecordScanCursor,
    width: u32,
    scratch_bytes: usize,
) -> Vec<(worth_store::physical_runtime::PhysicalRecordId, Vec<u8>)> {
    let scan = serving
        .records()
        .expect("read protection admission")
        .scan(
            RecordScanRequest::resume(cursor)
                .with_batch_limit(RecordCountLimit::new(width).unwrap()),
        )
        .unwrap();
    collect_session(scan, scratch_bytes).records
}

fn collect_session(
    mut scan: worth_store::physical_runtime::PhysicalRecordScanSession,
    scratch_bytes: usize,
) -> CollectedScan {
    let mut scratch = vec![0_u8; scratch_bytes];
    let mut found = Vec::new();
    let mut batches = Vec::new();
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
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

struct CollectedScan {
    records: Vec<(worth_store::physical_runtime::PhysicalRecordId, Vec<u8>)>,
    batches: Vec<RecordScanCounterSnapshot>,
}

fn assert_exact_batch_accounting(
    width: u32,
    payloads: &[Vec<u8>],
    batches: &[RecordScanCounterSnapshot],
) {
    const DURABLE_FRAME_HEADER_BYTES: u64 = super::durable_frame_oracle::HEADER_BYTES as u64;
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
