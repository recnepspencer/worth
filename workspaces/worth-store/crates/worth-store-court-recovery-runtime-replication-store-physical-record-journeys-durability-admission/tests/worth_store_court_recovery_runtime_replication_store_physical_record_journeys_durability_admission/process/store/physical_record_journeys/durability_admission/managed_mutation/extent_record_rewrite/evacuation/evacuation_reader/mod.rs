use {crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation::LIVE_PAYLOAD_BYTES, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation::REOPEN_DIRECTORY};
use {worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest};

#[test]
#[ignore = "the sparse-arena parent starts this reader in a fresh process"]
pub(super) fn evacuation_reader_child() {
    let directory = std::path::PathBuf::from(std::env::var_os(REOPEN_DIRECTORY).unwrap());
    let root = directory.join("store");
    let identity = std::fs::read(directory.join("evacuated-record.id")).unwrap();
    assert_eq!(identity.len(), 24);
    let reopened = crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement::reopen_with_copy_policy(&root);
    let mut scan = reopened
        .records()
        .unwrap()
        .scan(worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(32).unwrap()))
        .unwrap();
    let mut scratch = vec![0; 64 * 1024];
    let mut selected = None;
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for record in batch.records() {
            let id = record.record_id();
            if id.allocation_epoch().as_slice() == &identity[..16]
                && id.ordinal().to_le_bytes().as_slice() == &identity[16..24]
            {
                assert!(selected.replace(id).is_none(), "stable id is unique");
                assert_eq!(record.declared_payload_bytes(), LIVE_PAYLOAD_BYTES as u64);
            }
        }
    }
    drop(scan);
    let reader = reopened.records().unwrap();
    let session = reader
        .open(
            selected.expect("fresh-process scan finds the exact stable id"),
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(LIVE_PAYLOAD_BYTES as u32).unwrap()),
        )
        .unwrap();
    assert_eq!(
        worth_store_testkit_store::physical_record_journeys::read_record(session, LIVE_PAYLOAD_BYTES).0,
        vec![77; LIVE_PAYLOAD_BYTES]
    );
    drop(reader);
    reopened.close();
}
