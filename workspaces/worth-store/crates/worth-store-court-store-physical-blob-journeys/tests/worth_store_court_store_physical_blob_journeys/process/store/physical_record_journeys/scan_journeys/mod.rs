#[allow(unused_imports)] use worth_store_testkit_store::physical_record_journeys::scan_journeys::*;
use {worth_store_testkit_store::durable_publication::root::publish_single, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration, worth_store_testkit_store::physical_record_journeys::success};
use {worth_store::physical_runtime::ExternalRecordScanCursor, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordScanCounterSnapshot, worth_store::physical_runtime::RecordScanDenial, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest};




#[test]
fn scan_batch_widths_converge_to_one_physical_sequence() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(2);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
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
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter(payloads.iter()).unwrap(),
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
        .scan(worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(3).unwrap()))
        .unwrap();
    let mut scratch = vec![0_u8; 32_000];
    let cursor = match first.read_next_into(&mut scratch).unwrap() {
        worth_store::physical_runtime::RecordScanOutcome::Batch(batch) => {
            assert_eq!(batch.records().len(), 3);
            batch.end_cursor()
        }
        worth_store::physical_runtime::RecordScanOutcome::Completed(_) => panic!("the seeded scan cannot complete empty"),
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
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .transfer_limit(worth_store::physical_runtime::RecordByteLimit::new(65_536).unwrap())
        .scratch_limit(worth_store::physical_runtime::RecordByteLimit::new(131_072).unwrap())
        .admit(format)
        .unwrap();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let logical_bytes = 17 * 65_536 + 7;
    let batch = worth_store::physical_runtime::RecordAppendBatch::builder()
        .push_source(worth_store_testkit_store::stream_fixture::PatternSource::exact(logical_bytes))
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
        .scan(worth_store::physical_runtime::RecordScanRequest::from_start())
        .unwrap();
    let mut scratch = vec![0_u8; 131_072];
    let batch = match scan.read_next_into(&mut scratch).unwrap() {
        worth_store::physical_runtime::RecordScanOutcome::Batch(batch) => batch,
        worth_store::physical_runtime::RecordScanOutcome::Completed(_) => panic!("the extent record must be discoverable"),
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
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(logical_bytes as u32).unwrap()),
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
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([162; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"alpha".as_slice(), b"beta".as_slice()]).unwrap(),
    );
    let mut scan = serving
        .records()
        .expect("read protection admission")
        .scan(worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(1).unwrap()))
        .unwrap();
    let mut scratch = [0_u8; 16];
    let valid = match scan.read_next_into(&mut scratch).unwrap() {
        worth_store::physical_runtime::RecordScanOutcome::Batch(batch) => batch.end_cursor(),
        worth_store::physical_runtime::RecordScanOutcome::Completed(_) => unreachable!(),
    };
    drop(scan);

    for (offset, expected) in [
        (0, worth_store::physical_runtime::RecordScanDenial::ForeignStore),
        (16, worth_store::physical_runtime::RecordScanDenial::StaleRoot),
        (24, worth_store::physical_runtime::RecordScanDenial::RoutingTreeMismatch),
        (32, worth_store::physical_runtime::RecordScanDenial::FormatMismatch),
        (42, worth_store::physical_runtime::RecordScanDenial::CursorPositionNotFound),
    ] {
        let mut bytes = valid.encode();
        bytes[offset] ^= 0x7f;
        let forged = worth_store::physical_runtime::ExternalRecordScanCursor::decode(bytes).unwrap();
        let error = match serving
            .records()
            .expect("read protection admission")
            .scan(worth_store::physical_runtime::RecordScanRequest::resume(forged))
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
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([163; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"alpha".as_slice(), b"beta".as_slice()]).unwrap(),
    );
    let mut scan = serving
        .records()
        .expect("read protection admission")
        .scan(worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(1).unwrap()))
        .unwrap();
    let error = match scan.read_next_into(&mut []) {
        Ok(_) => panic!("empty scratch cannot hold the first record"),
        Err(error) => error,
    };
    assert_eq!(
        error.denial(),
        worth_store::physical_runtime::RecordScanDenial::CallerScratchTooSmall { required: 5 }
    );
    assert!(error.observation().manifest_blocks() > 0);
    let discovered = error.observation().manifest_blocks();
    let mut scratch = [0_u8; 5];
    let batch = match scan.read_next_into(&mut scratch).unwrap() {
        worth_store::physical_runtime::RecordScanOutcome::Batch(batch) => batch,
        worth_store::physical_runtime::RecordScanOutcome::Completed(_) => unreachable!(),
    };
    assert_eq!(batch.payload(0), Some(b"alpha".as_slice()));
    assert_eq!(batch.observation().manifest_blocks(), discovered + 1);
    drop(scan);
    serving.close();
}
#[path = "capped_payload/mod.rs"]
pub(crate) mod capped_payload;
#[path = "route_reopen/mod.rs"]
pub(crate) mod route_reopen;
