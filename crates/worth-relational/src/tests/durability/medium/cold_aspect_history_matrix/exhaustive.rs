use super::*;

#[test]
fn residency_failure_matrix_aspect_history_doors_cover_every_subject_and_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for subject in Subject::ALL {
        for door in Door::ALL {
            for (cause, sticky) in FAILURES {
                let fixture = CurrentFixture::reopened(execution);
                let targets = subject.targets(&fixture);
                assert!(!targets.is_empty());
                let frontier = fixture.runtime.history().latest_patch_stream_position();
                let offered = attached_test_medium(&fixture.runtime).offered().len();
                let diagnostics = fixture.runtime.publication.diagnostics.snapshot();
                fixture
                    .source
                    .failures
                    .lock()
                    .unwrap()
                    .extend(targets.iter().map(|target| (*target, cause)));
                let denial = door.run(execution, &fixture).unwrap_err();
                let Execution::StorageUnavailable(failure) = denial else {
                    panic!("{subject:?} x {door:?} x {cause:?}: {denial:?}");
                };
                assert_eq!(failure.cause, cause);
                subject.assert_subject(&fixture, failure);
                fixture.source.failures.lock().unwrap().clear();
                let before = attempts(&fixture, &targets);
                if sticky {
                    assert_eq!(door.run(execution, &fixture), Err(denial));
                    assert_eq!(
                        attempts(&fixture, &targets),
                        before,
                        "{subject:?} x {door:?}: only the failed subject must stay damaged"
                    );
                } else {
                    assert!(
                        door.run(execution, &fixture)
                            .unwrap_or_else(|failure| panic!(
                                "retry {subject:?} x {door:?} x {cause:?}: {failure:?}"
                            ))
                            > 0
                    );
                    assert!(attempts(&fixture, &targets) > before);
                }
                assert_eq!(
                    fixture.runtime.history().latest_patch_stream_position(),
                    frontier
                );
                assert_eq!(
                    attached_test_medium(&fixture.runtime).offered().len(),
                    offered
                );
                assert_eq!(
                    fixture.runtime.publication.diagnostics.snapshot(),
                    diagnostics
                );
            }
        }
    }
}
