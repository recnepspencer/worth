//! Declared Head operations preserve native typed failure for each subject.
use super::*;
use crate::storage::data::RelationalStorageSubject;
#[derive(Debug, Clone, Copy)]
enum Subject {
    Head,
    Symbols,
}
#[test]
fn residency_failure_matrix_declared_current_doors_preserve_every_failure_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for subject in [Subject::Head, Subject::Symbols] {
        for door in CurrentDoor::ALL {
            for (cause, sticky) in FAILURES {
                let fixture = CurrentFixture::reopened(execution);
                let members = match subject {
                    Subject::Head => &fixture.heads,
                    Subject::Symbols => &fixture.symbols,
                };
                assert!(!members.is_empty());
                let frontier = fixture.runtime.history().latest_patch_stream_position();
                let offered = attached_test_medium(&fixture.runtime).offered().len();
                let diagnostics = fixture.runtime.publication.diagnostics.snapshot();
                fixture.source.failures.lock().unwrap().extend(
                    members
                        .iter()
                        .map(|member| (ColdRead::Member(*member), cause)),
                );
                let run = || {
                    door.run(
                        execution,
                        &fixture.runtime,
                        fixture.entity,
                        fixture.relation,
                    )
                };
                let failure = run().unwrap_err();
                assert_eq!(failure.cause, cause, "{subject:?} x {door:?}");
                match subject {
                    Subject::Head => assert_eq!(
                        failure.subject,
                        RelationalStorageSubject::Region(fixture.entity.partition_id)
                    ),
                    Subject::Symbols => assert!(matches!(
                        failure.subject,
                        RelationalStorageSubject::Symbols { .. }
                    )),
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
                fixture.source.failures.lock().unwrap().clear();
                let reads = fixture.source.reads.load(Ordering::Relaxed);
                if sticky {
                    assert_eq!(run(), Err(failure));
                    assert!(fixture.source.member_reads.lock().unwrap()[reads..].iter()
                        .all(|member| !members.contains(member)),
                        "{subject:?} x {door:?} x {cause:?}: proven damage never rereads its members");
                    assert_eq!(
                        fixture.runtime.history().latest_patch_stream_position(),
                        frontier
                    );
                    assert_eq!(
                        attached_test_medium(&fixture.runtime).offered().len(),
                        offered
                    );
                } else {
                    run().unwrap_or_else(|failure| {
                        panic!("retry {subject:?} x {door:?} x {cause:?}: {failure:?}")
                    });
                    assert!(fixture.source.reads.load(Ordering::Relaxed) > reads);
                    if door.publishes() {
                        assert!(
                            fixture.runtime.history().latest_patch_stream_position() > frontier
                        );
                        assert!(attached_test_medium(&fixture.runtime).offered().len() > offered);
                    } else {
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
    }
}
