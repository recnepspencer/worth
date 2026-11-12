//! Real node-page point reads and sticky typed failures through shipping consumers.
use super::{lineage_node_doors::NodeDoor, lineage_node_source::reopened};
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    durability::RelationalCanonicalCommitArtifact,
    identity::LineageId,
    storage::{
        RelationalStorageSubject, RelationalStorageUnavailable,
        RelationalStorageUnavailableCause as Cause,
    },
    transactions::RelationalExecutionDenialCause,
};
#[test]
fn a_cold_lineage_point_read_loads_one_node_page_and_no_region_history() {
    let backing = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let fixture = reopened(execution);
    let commits = fixture.source.commit_reads.lock().unwrap().len();
    let (_, basis) = fixture
        .runtime
        .observe_branch(&fixture.runtime.main_branch_identity())
        .unwrap();
    let observed = fixture
        .runtime
        .lineage_node_at_observation(
            execution,
            &basis.observation(),
            fixture.expected.entity_id(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(observed.node(), &fixture.expected);
    let reads = fixture.source.member_reads.lock().unwrap();
    assert_eq!(
        reads
            .iter()
            .filter(|locator| **locator == fixture.page.member)
            .count(),
        1
    );
    assert!(
        !reads.contains(
            &fixture
                .directory
                .mirror
                .iter()
                .find(|r| r.partition_id == fixture.expected.entity_id().partition_id)
                .unwrap()
                .history
        ),
        "record association requires Head only"
    );
    let selected = fixture
        .runtime
        .history()
        .branch_head_for_observation(&basis.observation())
        .unwrap()
        .unwrap();
    let selected_history = fixture
        .directory
        .roots
        .iter()
        .find(|root| root.commit_id == selected.commit_id)
        .unwrap()
        .regions
        .iter()
        .find(|region| region.partition_id == fixture.expected.entity_id().partition_id)
        .unwrap()
        .history;
    assert!(
        !reads.contains(&selected_history),
        "the actual selected root requires Head only"
    );
    assert_eq!(
        fixture.source.commit_reads.lock().unwrap().len(),
        commits,
        "node lookup does not replay envelopes"
    );
}

#[test]
fn every_node_guard_preserves_a_sticky_typed_page_failure_without_publication() {
    let backing = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    for door in NodeDoor::ALL {
        if matches!(
            door,
            NodeDoor::MemoryCheckpoint | NodeDoor::EmbeddedCheckpoint
        ) {
            continue;
        }
        let fixture = reopened(execution);
        let (_, basis) = fixture
            .runtime
            .observe_branch(&fixture.runtime.main_branch_identity())
            .unwrap();
        assert!(fixture
            .runtime
            .read_truth()
            .project_observation(&basis.observation())
            .unwrap()
            .read_exact_entity_state(execution, fixture.expected.entity_id(), |_, _| ())
            .unwrap()
            .is_some());
        // Fund the real canonical bodies through their ordinary public history owner.
        for bytes in fixture.source.commits.values() {
            let artifact =
                RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
            let receipt = fixture
                .runtime
                .history()
                .immutable_commit_receipt_at_patch_stream_position(artifact.position())
                .unwrap();
            assert!(fixture
                .runtime
                .history()
                .historical_committed_version(execution, receipt.version_id)
                .unwrap()
                .is_some());
        }
        let position = fixture.runtime.history().latest_patch_stream_position();
        let offered = fixture.medium.offered().len();
        *fixture.source.failed_member.lock().unwrap() = Some(fixture.page.member);
        let expected =
            RelationalExecutionDenialCause::StorageUnavailable(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Lineage {
                    first: LineageId(fixture.page.first),
                    last: LineageId(fixture.page.last),
                },
                cause: Cause::Damaged,
            });
        assert_eq!(door.read(&fixture, execution), Err(expected), "{door:?}");
        let reads = fixture.source.reads.load(Ordering::Relaxed);
        *fixture.source.failed_member.lock().unwrap() = None;
        assert_eq!(door.read(&fixture, execution), Err(expected), "{door:?}");
        assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
        assert_eq!(
            fixture.runtime.history().latest_patch_stream_position(),
            position
        );
        assert_eq!(fixture.medium.offered().len(), offered);
    }
}
