use super::*;

#[test]
fn residency_failure_matrix_lineage_event_doors_preserve_every_failure_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in EventDoor::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = reopened(execution);
            let directory = fixture
                .runtime
                .durability
                .checkpoint_selection()
                .snapshot()
                .unwrap();
            let page = directory
                .manifest
                .lineage_events
                .iter()
                .find(|page| {
                    page.event_ids
                        .iter()
                        .any(|id| fixture.expected.traversed_event_ids.contains(id))
                })
                .unwrap();
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = attached_test_medium(&fixture.runtime).offered().len();
            fixture
                .source
                .failures
                .lock()
                .unwrap()
                .insert(ColdRead::Member(page.member), cause);
            let expected = Execution::StorageUnavailable(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Lineage {
                    first: crate::identity::data::LineageId(page.first_node),
                    last: crate::identity::data::LineageId(page.last_node),
                },
                cause,
            });
            assert_eq!(
                door.read(&fixture, execution),
                Err(expected),
                "{door:?} x {cause:?}"
            );
            fixture.source.failures.lock().unwrap().clear();
            let reads = || {
                fixture
                    .source
                    .member_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|member| **member == page.member)
                    .count()
            };
            let before = reads();
            if sticky {
                assert_eq!(door.read(&fixture, execution), Err(expected), "{door:?}");
                assert_eq!(
                    EventDoor::Historical.read(&fixture, execution),
                    Err(expected),
                    "export shares the same damage cell"
                );
                assert_eq!(reads(), before);
            } else {
                assert!(
                    door.read(&fixture, execution).unwrap() > 0,
                    "retry must return genuine history"
                );
                assert!(reads() > before);
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
