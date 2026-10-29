use super::*;

#[test]
fn residency_failure_matrix_current_doors_never_consult_region_history() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in CurrentDoor::ALL {
        for (cause, _) in FAILURES {
            let fixture = CurrentFixture::reopened(execution);
            assert!(!fixture.history.is_empty());
            fixture.source.failures.lock().unwrap().extend(
                fixture
                    .history
                    .iter()
                    .map(|member| (ColdRead::Member(*member), cause)),
            );
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = attached_test_medium(&fixture.runtime).offered().len();
            door.run(
                execution,
                &fixture.runtime,
                fixture.entity,
                fixture.relation,
            )
            .unwrap();
            assert!(
                !fixture
                    .source
                    .member_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|member| fixture.history.contains(member)),
                "{door:?} x {cause:?} opened history"
            );
            if matches!(door, CurrentDoor::Commit | CurrentDoor::DeleteRelation) {
                assert!(fixture.runtime.history().latest_patch_stream_position() > frontier);
                assert!(attached_test_medium(&fixture.runtime).offered().len() > offered);
                let mut transaction = test_owner_begin_transaction_for_main(&fixture.runtime);
                assert_eq!(
                    transaction
                        .read_entity(execution, fixture.entity)
                        .unwrap()
                        .effective()
                        .unwrap()
                        .entity_id,
                    fixture.entity
                );
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
