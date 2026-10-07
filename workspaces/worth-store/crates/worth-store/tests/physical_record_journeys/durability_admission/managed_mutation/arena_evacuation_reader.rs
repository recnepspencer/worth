use super::{LIVE_PAYLOAD_BYTES, REOPEN_DIRECTORY};
use worth_store::physical_runtime::{
    RecordByteLimit, RecordCountLimit, RecordReadLimits, RecordScanOutcome, RecordScanRequest,
};

#[test]
#[ignore = "the sparse-arena parent starts this reader in a fresh process"]
pub(super) fn evacuation_reader_child() {
    let directory = std::path::PathBuf::from(std::env::var_os(REOPEN_DIRECTORY).unwrap());
    let root = directory.join("store");
    let identity = std::fs::read(directory.join("evacuated-record.id")).unwrap();
    assert_eq!(identity.len(), 24);
    let reopened = super::super::arena_retirement::reopen_with_copy_policy(&root);
    let mut scan = reopened
        .records()
        .unwrap()
        .scan(RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(32).unwrap()))
        .unwrap();
    let mut scratch = vec![0; 64 * 1024];
    let mut selected = None;
    loop {
        match scan.read_next_into(&mut scratch).unwrap() {
            RecordScanOutcome::Batch(batch) => {
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
            RecordScanOutcome::Completed(_) => break,
        }
    }
    drop(scan);
    let reader = reopened.records().unwrap();
    let session = reader
        .open(
            selected.expect("fresh-process scan finds the exact stable id"),
            RecordReadLimits::new(RecordByteLimit::new(LIVE_PAYLOAD_BYTES as u32).unwrap()),
        )
        .unwrap();
    assert_eq!(
        crate::read_record(session, LIVE_PAYLOAD_BYTES).0,
        vec![77; LIVE_PAYLOAD_BYTES]
    );
    drop(reader);
    reopened.close();
}
