//! Public Full read guards admit meanings even when another fork warmed bytes.
use super::super::cold_visibility::ReadDoor;
use super::view_levels::reopened;
use super::*;
use crate::execution::RelationalExecutionDenialCause;

#[test]
fn every_full_read_door_admits_its_runtime_dictionary_or_preserves_page_damage() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in ReadDoor::ALL {
        for damage in [false, true] {
            let fixture = reopened(execution);
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            if damage {
                *fixture.source.failed_member.lock().unwrap() = Some(fixture.historical_page);
                let failure = door
                    .run(execution, &fixture.runtime, fixture.entity)
                    .unwrap_err();
                assert!(
                    matches!(failure, RelationalExecutionDenialCause::StorageUnavailable(
                        RelationalStorageUnavailable {
                            subject: RelationalStorageSubject::Symbols { first, last },
                            cause: Cause::Damaged,
                        }
                    ) if first <= fixture.historical && fixture.historical <= last),
                    "{door:?}: {failure:?}"
                );
                *fixture.source.failed_member.lock().unwrap() = None;
                let reads = fixture.source.reads.load(Ordering::Relaxed);
                assert_eq!(
                    door.run(execution, &fixture.runtime, fixture.entity)
                        .unwrap_err(),
                    failure,
                    "{door:?}"
                );
                assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
                let mut transaction = test_owner_begin_transaction_for_main(&fixture.runtime);
                assert_eq!(
                    transaction
                        .read_entity(execution, fixture.entity)
                        .unwrap()
                        .effective()
                        .unwrap()
                        .entity_id,
                    fixture.entity,
                    "a history-only page failure must not poison Head: {door:?}"
                );
            } else {
                door.run(execution, &fixture.runtime, fixture.entity)
                    .unwrap();
                assert_eq!(
                    fixture
                        .runtime
                        .services
                        .symbols
                        .interner_snapshot()
                        .resolve(fixture.historical),
                    Some("paged-view-value-0300"),
                    "{door:?} must admit Full meanings into its own runtime"
                );
            }
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier
            );
        }
    }
}
