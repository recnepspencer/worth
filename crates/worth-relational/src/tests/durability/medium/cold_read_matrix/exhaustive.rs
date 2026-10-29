use super::*;

#[test]
fn residency_failure_matrix_full_read_subjects_cover_every_door_and_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for subject in ReadSubject::ALL {
        for door in ReadDoor::ALL {
            for (cause, sticky) in FAILURES {
                let fixture = MatrixFixture::new(subject, execution);
                door.prepare(&fixture.runtime);
                let frontier = fixture.runtime.history().latest_patch_stream_position();
                let offered = attached_test_medium(&fixture.runtime).offered().len();
                let target_reads_before = fixture
                    .source
                    .member_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|member| fixture.targets.contains(member))
                    .count();
                fixture.inject(cause);
                if door.is_current_projection_scan() && !matches!(subject, ReadSubject::RegionHead)
                {
                    fixture.run(door, execution).unwrap_or_else(|failure| {
                        panic!("current scan touched {subject:?}: {failure:?}")
                    });
                    assert_eq!(
                        fixture
                            .source
                            .member_reads
                            .lock()
                            .unwrap()
                            .iter()
                            .filter(|member| fixture.targets.contains(member))
                            .count(),
                        target_reads_before,
                        "current scans open no additional historical member"
                    );
                    if let Some(symbol) = fixture.expected_symbol {
                        assert!(fixture
                            .runtime
                            .services
                            .symbols
                            .interner_snapshot()
                            .resolve(symbol)
                            .is_none());
                    }
                    assert_eq!(
                        fixture.runtime.history().latest_patch_stream_position(),
                        frontier
                    );
                    assert_eq!(
                        attached_test_medium(&fixture.runtime).offered().len(),
                        offered
                    );
                    continue;
                }
                let denial = fixture.run(door, execution).unwrap_err();
                let RelationalExecutionDenialCause::StorageUnavailable(failure) = denial else {
                    panic!("{subject:?}  x  {door:?}  x  {cause:?}: {denial:?}");
                };
                assert_eq!(failure.cause, cause, "{subject:?}  x  {door:?}");
                fixture.assert_subject(failure);
                fixture.source.failures.lock().unwrap().clear();
                let reads = fixture.source.reads.load(Ordering::Relaxed);
                let failed_reads = || {
                    fixture
                        .source
                        .member_reads
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|member| fixture.targets.contains(member))
                        .count()
                };
                let failed_before = failed_reads();
                if sticky {
                    assert_eq!(
                        fixture.run(door, execution),
                        Err(denial),
                        "{subject:?}  x  {door:?}  x  {cause:?}"
                    );
                    if matches!(
                        door,
                        ReadDoor::MemoryCheckpoint | ReadDoor::EmbeddedCheckpoint
                    ) {
                        assert_eq!(failed_reads(), failed_before);
                    } else {
                        assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
                    }
                } else {
                    fixture.run(door, execution).unwrap_or_else(|failure| {
                        panic!("retry {subject:?}  x  {door:?}  x  {cause:?}: {failure:?}")
                    });
                    assert!(fixture.source.reads.load(Ordering::Relaxed) > reads);
                }
                assert_eq!(
                    fixture.runtime.history().latest_patch_stream_position(),
                    frontier
                );
                assert_eq!(
                    attached_test_medium(&fixture.runtime).offered().len(),
                    offered
                );
            }
        }
    }
}
