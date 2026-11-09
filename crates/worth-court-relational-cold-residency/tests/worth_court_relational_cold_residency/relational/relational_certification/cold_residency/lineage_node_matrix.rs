use super::lineage_event_matrix::FAILURES;
use super::{lineage_node_doors::NodeDoor, lineage_node_source::reopened};
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    identity::LineageId,
    storage::{RelationalStorageSubject, RelationalStorageUnavailable},
    transactions::RelationalExecutionDenialCause as Execution,
};
use worth_testkit_relational::durable_medium::immutable_members::ColdRead;

#[test]
fn residency_failure_matrix_lineage_doors_preserve_every_failure_class() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in NodeDoor::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = reopened(execution);
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = fixture.medium.offered().len();
            fixture
                .source
                .failures
                .lock()
                .unwrap()
                .insert(ColdRead::Member(fixture.page.member), cause);
            let expected = Execution::StorageUnavailable(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Lineage {
                    first: LineageId(fixture.page.first),
                    last: LineageId(fixture.page.last),
                },
                cause,
            });
            assert_eq!(
                door.read(&fixture, execution),
                Err(expected),
                "{door:?} x {cause:?}"
            );
            fixture.source.failures.lock().unwrap().clear();
            let reads = fixture.source.reads.load(Ordering::Relaxed);
            let failed_reads = || {
                fixture
                    .source
                    .member_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|member| **member == fixture.page.member)
                    .count()
            };
            let failed_before = failed_reads();
            if sticky {
                assert_eq!(door.read(&fixture, execution), Err(expected));
                if matches!(
                    door,
                    NodeDoor::MemoryCheckpoint | NodeDoor::EmbeddedCheckpoint
                ) {
                    assert_eq!(failed_reads(), failed_before);
                } else {
                    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
                }
            } else {
                door.read(&fixture, execution).unwrap();
                assert!(fixture.source.reads.load(Ordering::Relaxed) > reads);
                if matches!(door, NodeDoor::EmbeddedCheckpoint) {
                    let (_, basis) = fixture
                        .runtime
                        .observe_branch(&fixture.runtime.main_branch_identity())
                        .unwrap();
                    assert_eq!(
                        fixture
                            .runtime
                            .lineage_node_at_observation(
                                execution,
                                &basis.observation(),
                                fixture.expected.entity_id()
                            )
                            .unwrap()
                            .as_ref()
                            .map(|read| read.node()),
                        Some(&fixture.expected)
                    );
                }
            }
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier
            );
            assert_eq!(fixture.medium.offered().len(), offered);
        }
    }
}
