use super::*;
#[test]
fn protected_payload_stream_rejects_actual_truncated_selected_record() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let serving = initialize(root);
    publish_anchor(&serving, 0, vec![41; 97]);
    serving.close();
    // A small genuine anchor is an inline record in the first segment.
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(
            root.join("families/records/segments/segment-0000000000000001-0000000000000001.pages"),
        )
        .unwrap();
    assert!(file.metadata().unwrap().len() > 0);
    file.set_len(0).unwrap();
    drop(file);
    let serving = reopen(root);
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        cancellation.token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let reader = serving.try_records_for_checkpoint(&request).unwrap();
    assert!(
        matches!(serving.anchored_records().unwrap().open_payload_from(&reader, &request, anchor(), 1 << 20),
        Err(AnchoredRecordReadFailure::Open(error)) if error.denial() == RecordReadDenial::ArtifactDamaged)
    );
    drop(reader);
    assert!(serving.close_plan().execute().requires_inspection());
}
