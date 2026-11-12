//! Current read and preparation owners use Head without historical fallback.
use super::current_field_meanings::*;
use super::current_read_basis::*;
use super::{current_publications, current_source};
use current_door::CurrentDoor;
use current_publications::*;
use current_source::{capture_and_reopen, CurrentFixture};
use std::sync::atomic::Ordering;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_relational::facade::{durability::*, history::BranchId, identity::*};
use worth_testkit_relational::{
    aspect_storage::publications::{
        begin_main_transaction, changed_entities, release_test_commit_snapshot,
    },
    cold_recovery::update_entity,
    durable_medium::{immutable_members::ColdRead, TestDurableMedium},
    durable_publication::{create_entity_outcome, durable_runtime, memory_runtime},
};
const FAILURES: [(Cause, bool); 6] = [
    (Cause::MissingMember, true),
    (Cause::SourceUnavailable, false),
    (Cause::Damaged, true),
    (Cause::IntegrityMismatch, true),
    (
        Cause::LimitExceeded {
            length: 2,
            limit: 1,
        },
        false,
    ),
    (Cause::MemoryExhausted, false),
];
#[path = "accepted_pins.rs"]
mod accepted_pins;
#[path = "adjacency_support.rs"]
mod adjacency_support;
#[path = "aspect_history_matrix.rs"]
mod aspect_history_matrix;
#[path = "current_door.rs"]
mod current_door;
#[path = "current_failure_matrix.rs"]
mod current_failure_matrix;
#[path = "current_success.rs"]
mod current_success;
#[path = "metadata_matrix.rs"]
mod metadata_matrix;
#[path = "projection_point_door.rs"]
mod projection_point_door;
#[path = "snapshot_point_door.rs"]
mod snapshot_point_door;
#[path = "storage_adjacency.rs"]
mod storage_adjacency;
#[test]
fn head_write_capture_and_full_promotion_preserve_current_and_retained_reference_reads() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = CurrentFixture::reopened(execution);
    let (_, before) = fixture
        .runtime
        .observe_branch(&fixture.runtime.main_branch_identity())
        .unwrap();
    let observation = before.observation();
    let reference = memory_runtime();
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
    update_entity(execution, &reference, expected_entity, "head-only-current");
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
            execution,
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
            main_selected_version(&fixture.runtime),
        )
        .unwrap_err();
    assert!(
        matches!(failure, worth_relational::facade::transactions::RelationalExecutionDenialCause::StorageUnavailable(
        worth_relational::facade::storage::RelationalStorageUnavailable {subject: worth_relational::facade::storage::RelationalStorageSubject::Region(partition), cause: Cause::SourceUnavailable}) if partition == fixture.entity.partition_id)
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
            main_selected_version(&fixture.runtime),
        )
        .unwrap()
        .unwrap();
    let expected = reference
        .read_truth()
        .visible_entity_at_version(
            execution,
            expected_entity,
            main_selected_version(&reference),
        )
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
                main_selected_version(&fixture.runtime)
            )
            .unwrap()
            .unwrap(),
        expected
    );
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
}

#[path = "materialization_doors.rs"]
mod materialization_doors;
#[path = "metadata_pins.rs"]
mod metadata_pins;
#[path = "retirement_metadata.rs"]
mod retirement_metadata;
