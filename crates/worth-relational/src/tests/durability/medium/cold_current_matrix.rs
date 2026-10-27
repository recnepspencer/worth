//! Current operations must not depend on unavailable historical payloads.
use super::cold_read_matrix::FAILURES;
use super::cold_source::ColdRead;
use super::*;
use crate::identity::data::{EntityId, RelationId};

#[path = "cold_aspect_history_matrix.rs"]
mod aspect_history_matrix;
#[path = "cold_current_door.rs"]
mod current_door;
#[path = "cold_current_fixture.rs"]
mod current_fixture;
#[path = "cold_current_failure_matrix.rs"]
mod failure_matrix;
#[path = "cold_projection_point_door.rs"]
mod projection_point_door;
use current_door::CurrentDoor;
use current_fixture::CurrentFixture;

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

#[test]
fn head_write_capture_and_full_promotion_preserve_current_and_retained_reference_reads() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = CurrentFixture::reopened(execution);
    let (_, before) = fixture
        .runtime
        .observe_branch(&fixture.runtime.main_branch_identity())
        .unwrap();
    let observation = before.observation();
    let reference = runtime_with_test_schema();
    let expected_entity = create_entity(execution, &reference, "head-only-before");
    let expected_target = create_entity(execution, &reference, "head-only-target");
    let linked = create_relation_outcome(
        execution,
        &reference,
        expected_entity,
        expected_target,
        "head-only-edge",
    );
    let expected_relation = changed_relations(&linked)[0];
    release_test_commit_snapshot(&reference, &linked);
    update_entity_and_release_snapshot(execution, &reference, expected_entity, "head-only-current");
    let (_, expected_before) = reference
        .observe_branch(&reference.main_branch_identity())
        .unwrap();
    let expected_old = reference
        .read_truth()
        .read_observation(execution, &expected_before.observation())
        .unwrap()
        .unwrap();
    fixture.source.failures.lock().unwrap().extend(
        fixture
            .history
            .iter()
            .map(|member| (ColdRead::Member(*member), Cause::SourceUnavailable)),
    );
    CurrentDoor::Commit
        .run(
            execution,
            &fixture.runtime,
            fixture.entity,
            fixture.relation,
        )
        .unwrap();
    CurrentDoor::Commit
        .run(execution, &reference, expected_entity, expected_relation)
        .unwrap();
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    let members = fixture.source.members.keys().copied().collect::<Vec<_>>();
    let checkpoint = fixture
        .runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &members,
                source: Some(fixture.source.clone()),
            },
        )
        .unwrap();
    assert!(!checkpoint.members().is_empty());
    assert_eq!(
        fixture.source.reads.load(Ordering::Relaxed),
        reads,
        "Head capture cannot load cold contents"
    );
    let frontier = fixture.runtime.history().latest_patch_stream_position();
    let failure = fixture
        .runtime
        .read_truth()
        .visible_entity_at_version(
            execution,
            fixture.entity,
            fixture.runtime.current_version_id(),
        )
        .unwrap_err();
    assert!(
        matches!(failure, crate::execution::RelationalExecutionDenialCause::StorageUnavailable(
        crate::storage::data::RelationalStorageUnavailable {subject: crate::storage::data::RelationalStorageSubject::Region(partition), cause: Cause::SourceUnavailable}) if partition == fixture.entity.partition_id)
    );
    assert_eq!(
        fixture.runtime.history().latest_patch_stream_position(),
        frontier
    );
    fixture.source.failures.lock().unwrap().clear();
    let full = fixture
        .runtime
        .read_truth()
        .visible_entity_at_version(
            execution,
            fixture.entity,
            fixture.runtime.current_version_id(),
        )
        .unwrap()
        .unwrap();
    let expected = reference
        .read_truth()
        .visible_entity_at_version(execution, expected_entity, reference.current_version_id())
        .unwrap()
        .unwrap();
    assert_eq!(full, expected);
    let retained = fixture
        .runtime
        .read_truth()
        .read_observation(execution, &observation)
        .unwrap()
        .unwrap();
    assert_eq!(retained.entities(), expected_old.entities());
    assert_eq!(retained.relations(), expected_old.relations());
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    assert_eq!(
        fixture
            .runtime
            .read_truth()
            .visible_entity_at_version(
                execution,
                fixture.entity,
                fixture.runtime.current_version_id()
            )
            .unwrap()
            .unwrap(),
        expected
    );
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
}
