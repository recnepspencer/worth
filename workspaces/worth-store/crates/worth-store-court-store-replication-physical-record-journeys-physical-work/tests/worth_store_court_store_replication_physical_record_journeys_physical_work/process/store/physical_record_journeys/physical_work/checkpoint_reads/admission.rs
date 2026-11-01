use super::*;
use sha2::{Digest, Sha256};
#[test]
fn checkpoint_reader_and_streams_refuse_busy_foreign_limits_and_original_stop() {
    let directory = tempfile::tempdir().unwrap();
    let foreign_directory = tempfile::tempdir().unwrap();
    let serving = initialize(directory.path());
    let foreign = initialize(foreign_directory.path());
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        cancellation.token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    publish_journals(&serving, 1, &[19; 100_000]);
    let member = b"actual member".to_vec();
    serving
        .anchored_records()
        .unwrap()
        .publish(
            AnchoredRecordPublication::new(
                anchor(),
                0,
                vec![31; 100_000],
                fixture::placement(),
                deadline(),
            )
            .with_members(vec![AnchoredRecordMember::New {
                key: Sha256::digest(&member).into(),
                bytes: member,
            }]),
        )
        .unwrap();
    let gate = serving.certification_pause_next_read_root_capture(
        certification::CertificationReadRootCaptureStage::AfterObservationBeforeRegistration,
    );
    std::thread::scope(|scope| {
        let paused = scope.spawn(|| serving.records().unwrap());
        assert!(gate.await_arrival());
        assert!(matches!(
            serving.try_records_for_checkpoint(&request),
            Err(PhysicalCheckpointReadProtectionDenial::Busy)
        ));
        gate.release();
        drop(paused.join().unwrap());
    });
    let reader = serving.try_records_for_checkpoint(&request).unwrap();
    let foreign_reader = foreign.try_records_for_checkpoint(&request).unwrap();
    let records = serving.anchored_records().unwrap();
    let journal = serving
        .ordered_journal(OrderedJournalName::new(JOURNALS[0]).unwrap())
        .unwrap();
    let before = serving.media_counters();
    assert!(matches!(
        records.open_payload_from(&foreign_reader, &request, anchor(), 1 << 20),
        Err(AnchoredRecordReadFailure::ForeignReader)
    ));
    assert!(matches!(
        journal.open_payload_from(&foreign_reader, &request, 1, 1 << 20),
        Err(OrderedJournalReadFailure::ForeignReader)
    ));
    assert_eq!(
        serving.media_counters(),
        before,
        "foreign reader rejected before I/O"
    );
    assert!(matches!(
        records.open_payload_from(&reader, &request, anchor(), 1),
        Err(AnchoredRecordReadFailure::LimitExceeded {
            payload_bytes: 100_000
        })
    ));
    assert!(matches!(
        journal.open_payload_from(&reader, &request, 1, 1),
        Err(OrderedJournalReadFailure::LimitExceeded {
            payload_bytes: 100_000
        })
    ));
    let exhausted =
        SerialRequest::from_memory(SerialMemoryBudget::new(Some(0)), cancellation.token(), None);
    assert!(matches!(
        records.open_payload_from(
            &reader,
            &ExecutionRequest::serial(&exhausted),
            anchor(),
            1 << 20
        ),
        Err(AnchoredRecordReadFailure::Execution(_))
    ));
    let mut empty_journal = journal
        .open_payload_from(&reader, &request, 1, 1 << 20)
        .unwrap();
    assert!(matches!(
        empty_journal.read_next(&mut []),
        Err(OrderedJournalReadFailure::EmptyReadBuffer)
    ));
    assert!(matches!(
        empty_journal.read_next(&mut [0; 1]),
        Err(OrderedJournalReadFailure::PreviouslyRefused)
    ));
    drop(empty_journal);
    let mut empty = records
        .open_payload_from(&reader, &request, anchor(), 1 << 20)
        .unwrap()
        .unwrap();
    assert!(matches!(
        empty.read_next(&mut []),
        Err(AnchoredRecordReadFailure::EmptyReadBuffer)
    ));
    assert!(matches!(
        empty.read_next(&mut [0; 1]),
        Err(AnchoredRecordReadFailure::PreviouslyRefused)
    ));
    assert_eq!(empty.verified_version(), None);
    let mut anchor_stream = records
        .open_payload_from(&reader, &request, anchor(), 1 << 20)
        .unwrap()
        .unwrap();
    let mut journal_stream = journal
        .open_payload_from(&reader, &request, 1, 1 << 20)
        .unwrap();
    assert_eq!(anchor_stream.read_next(&mut [0; 7]).unwrap(), 7);
    assert_eq!(journal_stream.read_next(&mut [0; 7]).unwrap(), 7);
    cancellation.cancel();
    assert!(matches!(
        serving.try_records_for_checkpoint(&request),
        Err(PhysicalCheckpointReadProtectionDenial::Request(_))
    ));
    assert!(matches!(
        anchor_stream.read_next(&mut [0; 7]),
        Err(AnchoredRecordReadFailure::Execution(_))
    ));
    assert!(matches!(
        journal_stream.read_next(&mut [0; 7]),
        Err(OrderedJournalReadFailure::Execution(_))
    ));
    assert!(matches!(
        journal_stream.read_next(&mut [0; 7]),
        Err(OrderedJournalReadFailure::PreviouslyRefused)
    ));
    assert!(matches!(
        anchor_stream.read_next(&mut [0; 7]),
        Err(AnchoredRecordReadFailure::PreviouslyRefused)
    ));
    assert_eq!(anchor_stream.verified_version(), None);
    drop(journal_stream);
    drop(anchor_stream);
    drop(empty);
    drop(foreign_reader);
    drop(reader);
    foreign.close();
    serving.close();
}
