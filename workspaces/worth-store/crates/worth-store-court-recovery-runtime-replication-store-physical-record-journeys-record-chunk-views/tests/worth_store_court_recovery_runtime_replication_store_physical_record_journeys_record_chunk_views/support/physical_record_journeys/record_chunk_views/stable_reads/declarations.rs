pub(crate) use {std::num::NonZeroU32};

pub(crate) use {worth_store::physical_runtime::PhysicalReadProtectionDenial, worth_store::physical_runtime::PhysicalReadProtectionDisposition, worth_store::physical_runtime::PhysicalReadProtectionPolicy, worth_store::physical_runtime::PhysicalRecordId, worth_store::physical_runtime::PhysicalRecordScanSession, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordReadDenial, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest, worth_store::physical_runtime::RecordStreamFailureKind};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture};
pub(crate) use {crate::record_chunk_views::managed_append::append};

pub(crate) fn borrowed_shutdown(abort: bool) {
    let parent = tempfile::tempdir().unwrap();
    let (serving, placement) = worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::initialize(&parent.path().join("borrowed-shutdown"));
    let expected = worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::payload(2 * worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES + 5);
    let record = append(&serving, placement, 206, &expected);
    let observer = serving.read_protection_observer();
    let mut session = serving
        .records()
        .unwrap()
        .open(record, limits(expected.len()))
        .unwrap();
    let chunk = session.next_chunk().unwrap().unwrap();
    let shutdown = if abort {
        serving.abort().read_protection()
    } else {
        serving.close().read_protection()
    };
    assert_eq!(
        shutdown.disposition(),
        worth_store::physical_runtime::PhysicalReadProtectionDisposition::RetainedUntilReadersRelease
    );
    assert_eq!(shutdown.observation().live_acquisitions(), 1);
    assert!(observer.snapshot().revoked());
    assert_eq!(chunk.bytes(), &expected[..worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::CHUNK_PAYLOAD_BYTES]);
    let before = session.observation();
    assert!(
        matches!(session.next_chunk(), Err(error) if error.kind() == worth_store::physical_runtime::RecordStreamFailureKind::RuntimeReleased)
    );
    assert_eq!(session.observation(), before);
    drop(session);
    assert_eq!(observer.snapshot().live_acquisitions(), 0);
    assert_eq!(observer.snapshot().releases(), 1);
}

pub(crate) fn limits(bytes: usize) -> RecordReadLimits {
    worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(bytes as u32).unwrap())
}

pub(crate) fn scan_request() -> RecordScanRequest {
    worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(1).unwrap())
}

pub(crate) fn collect_ids(scan: &mut PhysicalRecordScanSession) -> Vec<PhysicalRecordId> {
    let mut scratch = vec![0; 65_536];
    let mut ids = Vec::new();
    loop {
        match scan.read_next_into(&mut scratch).unwrap() {
            worth_store::physical_runtime::RecordScanOutcome::Batch(batch) => {
                ids.extend(batch.records().iter().map(|record| record.record_id()))
            }
            worth_store::physical_runtime::RecordScanOutcome::Completed(_) => return ids,
        }
    }
}