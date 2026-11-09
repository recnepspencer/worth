//! Every ordinary Full reader preserves the same original history-only page refusal.
use super::{full_read_inventory::FullReadDoor as ReadDoor, symbol_view_source::SymbolViewRecipe};
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    storage::{
        RelationalStorageSubject, RelationalStorageUnavailable,
        RelationalStorageUnavailableCause as Cause,
    },
    transactions::RelationalExecutionDenialCause,
};
use worth_testkit_relational::aspect_storage::publications::begin_main_transaction;
#[test]
fn full_read_doors_preserve_history_symbol_refusals_and_current_head() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let recipe = SymbolViewRecipe::new(execution);
    // The original query admission claim applies to reads; opaque checkpoint
    // transport has its independent symbol failure cells in cold_read_matrix.
    for door in ReadDoor::ALL
        .into_iter()
        .filter(|door| !door.is_checkpoint())
    {
        for damage in [false, true] {
            let fixture = recipe.reopen(execution);
            door.prepare(&fixture.runtime);
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let history_reads_before = fixture
                .source
                .member_reads
                .lock()
                .unwrap()
                .iter()
                .filter(|member| **member == fixture.historical_page)
                .count();
            if door.is_current_projection_scan() {
                *fixture.source.failed_member.lock().unwrap() = Some(fixture.historical_page);
                door.run(execution, &fixture.runtime, fixture.entity)
                    .unwrap();
                assert_eq!(
                    fixture
                        .source
                        .member_reads
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|member| **member == fixture.historical_page)
                        .count(),
                    history_reads_before
                );
                assert_eq!(
                    fixture.runtime.history().latest_patch_stream_position(),
                    frontier
                );
                continue;
            }
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
                let mut transaction = begin_main_transaction(&fixture.runtime);
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
            }
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier
            );
        }
    }
}
