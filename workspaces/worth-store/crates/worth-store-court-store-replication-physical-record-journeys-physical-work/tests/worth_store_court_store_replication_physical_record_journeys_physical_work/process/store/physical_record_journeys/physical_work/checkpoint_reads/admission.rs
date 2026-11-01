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
    publish_anchor(&foreign, 0, b"metadata-only calibration".to_vec());
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
    assert!(matches!(
        journal.describe_entry_from(&foreign_reader, &request, 1),
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
    // Calibrate actual fixed metadata using a genuine selected empty table.
    // This isolates table funding from the earlier Arc admission.
    let before_calibration = serial.memory().unwrap().charged_memory_bytes();
    let calibration = foreign
        .anchored_records()
        .unwrap()
        .open_payload_from(&foreign_reader, &request, anchor(), 1 << 20)
        .unwrap()
        .unwrap();
    let shared_metadata = serial.memory().unwrap().charged_memory_bytes() - before_calibration;
    assert!(shared_metadata > 0);
    // Admission itself temporarily retains the SDK's actual execution scope.
    // Measure that real scope while fixed metadata is held, without guessing
    // its private layout or confusing scope refusal with table refusal.
    let mut metadata_admission_peak = 0;
    request
        .run_native_with_checkpoint(|_| {
            metadata_admission_peak =
                serial.memory().unwrap().charged_memory_bytes() - before_calibration;
        })
        .unwrap();
    assert!(metadata_admission_peak > shared_metadata);
    // Ordinary generic session disposal is explicit unbudgeted fixture cleanup.
    drop(calibration);
    assert_eq!(
        serial.memory().unwrap().charged_memory_bytes(),
        before_calibration
    );
    let table_request = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(metadata_admission_peak)),
        cancellation.token(),
        None,
    );
    let empty_table = foreign
        .anchored_records()
        .unwrap()
        .open_payload_from(
            &foreign_reader,
            &ExecutionRequest::serial(&table_request),
            anchor(),
            1 << 20,
        )
        .unwrap()
        .unwrap();
    drop(empty_table);
    let denied = match records.open_payload_from(
        &reader,
        &ExecutionRequest::serial(&table_request),
        anchor(),
        1 << 20,
    ) {
        Err(cause) => cause,
        Ok(_) => panic!("fixed metadata funding cannot impersonate funding the nonempty table"),
    };
    assert!(
        matches!(denied,
        AnchoredRecordReadFailure::Execution(worth_execution::WorkCeilingDenial::Admission(
            worth_execution::LeaseDenial::MemoryExhausted(worth_execution::MemoryLimitDenial::Capacity { requested, .. })
        )) if requested == std::mem::size_of::<AnchoredRecordMemberLocator>() as u64),
        "actual table denial: {denied:?}"
    );
    let exhausted =
        SerialRequest::from_memory(SerialMemoryBudget::new(Some(0)), cancellation.token(), None);
    let before_metadata_denial = serving.media_counters();
    assert!(matches!(
        records.open_payload_from(
            &reader,
            &ExecutionRequest::serial(&exhausted),
            anchor(),
            1 << 20
        ),
        Err(AnchoredRecordReadFailure::Execution(_))
    ));
    assert_eq!(
        serving.media_counters(),
        before_metadata_denial,
        "fixed metadata refusal precedes record I/O"
    );
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
    let empty_journal = foreign
        .ordered_journal(OrderedJournalName::new(JOURNALS[0]).unwrap())
        .unwrap();
    let before_absence = foreign.media_counters();
    assert!(matches!(
        empty_journal.describe_entry_from(&foreign_reader, &request, 1),
        Err(OrderedJournalReadFailure::Execution(_))
    ));
    assert_eq!(foreign.media_counters(), before_absence);
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
    let (anchor_stream, reader) = match anchor_stream.into_verified_membership(reader) {
        Err(same) => same,
        Ok(_) => panic!("a refused stream cannot issue verified membership"),
    };
    drop(anchor_stream);
    drop(empty);
    drop(foreign_reader);
    drop(reader);
    foreign.close();
    serving.close();
}
