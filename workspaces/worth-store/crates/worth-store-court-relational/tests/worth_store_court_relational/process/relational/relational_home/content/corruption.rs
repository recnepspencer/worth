use super::fixture::*;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    content::RelationalContentDenial, durability::RelationalRecoveryStop,
};
use worth_store::physical_runtime::*;
use worth_store_relational::RelationalStoreHomeOpenRefusal;

#[test]
fn content_missing_physical_member_refuses_reopen_before_service() {
    let host = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).unwrap();
    publish_grid(
        &home,
        &execution,
        worth_testkit_relational::content_grid::ExactSampleGrid {
            rows: 1,
            columns: 8192,
            samples: vec![0; 8192],
        },
        None,
    );
    home.checkpoint(&execution).unwrap();
    let records = home.certification_serving().anchored_records().unwrap();
    let anchor = AnchoredRecordName::new(crate::selected_checkpoint::ANCHOR).unwrap();
    let selected = records.read(anchor, 1 << 30).unwrap().unwrap();
    let content: Vec<_> = selected
        .members()
        .iter()
        .filter(|locator| {
            records
                .read_member(anchor, **locator, 1 << 30)
                .unwrap()
                .starts_with(b"WRTHCONT")
        })
        .copied()
        .collect();
    assert_eq!(content.len(), 1);
    records
        .publish(
            AnchoredRecordPublication::new(
                anchor,
                selected.version().generation(),
                selected.bytes().to_vec(),
                crate::open_refusals::home_placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .with_members(
                selected
                    .members()
                    .iter()
                    .filter(|locator| !content.contains(locator))
                    .map(|locator| AnchoredRecordMember::Carried { locator: *locator })
                    .collect(),
            ),
        )
        .unwrap();
    home.close();
    let refused = match open(&location, &execution) {
        Ok(_) => panic!("missing content must refuse before service"),
        Err(refusal) => refusal,
    };
    assert!(
        matches!(
            refused,
            RelationalStoreHomeOpenRefusal::Readmission(RelationalRecoveryStop::Content(
                RelationalContentDenial::MissingContent
            ))
        ),
        "{refused:?}"
    );
}

#[test]
fn content_damaged_store_data_refuses_before_native_service() {
    let host = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).unwrap();
    publish_grid(
        &home,
        &execution,
        worth_testkit_relational::content_grid::ExactSampleGrid {
            rows: 1,
            columns: 8192,
            samples: vec![0; 8192],
        },
        None,
    );
    home.checkpoint(&execution).unwrap();
    let records = home.certification_serving().anchored_records().unwrap();
    let anchor = AnchoredRecordName::new(crate::selected_checkpoint::ANCHOR).unwrap();
    let selected = records.read(anchor, 1 << 30).unwrap().unwrap();
    let bytes = selected
        .members()
        .iter()
        .find_map(|locator| {
            let bytes = records.read_member(anchor, *locator, 1 << 30).unwrap();
            bytes.starts_with(b"WRTHCONT").then_some(bytes)
        })
        .unwrap();
    home.close();
    let flipped: usize = std::fs::read_dir(location.join("families"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.file_name().unwrap() != "wal")
        .map(|path| crate::open_refusals::flip_after(&path, &bytes[..64], 63, &|_| true))
        .sum();
    assert!(flipped > 0);
    let refused = match open(&location, &execution) {
        Ok(_) => panic!("damaged content must refuse before service"),
        Err(refusal) => refusal,
    };
    // Store's physical frame checksum can refuse the covered journal before
    // Native sees the payload. The independent Native court separately repairs
    // the outer digest to exercise Native's own content integrity checker.
    assert!(
        matches!(&refused,
            RelationalStoreHomeOpenRefusal::Journal(worth_store_relational::RelationalStoreJournalRefusal::Unreadable { failure: OrderedJournalReadFailure::Stream(failure), .. })
                if failure.kind() == RecordStreamFailureKind::SelectedDataFrameChecksumDamaged
        ),
        "{refused:?}"
    );
}
