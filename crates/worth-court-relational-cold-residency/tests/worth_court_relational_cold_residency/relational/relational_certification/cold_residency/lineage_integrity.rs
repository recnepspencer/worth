//! Damage real producer bytes, independently from the provider error classifier.
use super::{lineage_event_doors::EventDoor, lineage_sources::reopened_with_source};
use worth_relational::facade::{
    identity::LineageId,
    storage::{
        RelationalStorageSubject, RelationalStorageUnavailable,
        RelationalStorageUnavailableCause as Cause,
    },
    transactions::RelationalExecutionDenialCause as Execution,
};
use worth_testkit_relational::durable_medium::changed_member::ChangedMemberSource;
#[test]
fn changed_event_bytes_damage_only_their_page_and_share_the_refusal_with_export() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for first in [EventDoor::Historical, EventDoor::EmbeddedCheckpoint] {
        let mut changed = None;
        let fixture = reopened_with_source(execution, |producer| {
            let source = ChangedMemberSource::new(producer);
            changed = Some(source.clone());
            source
        });
        let source = changed.unwrap();
        let page = fixture
            .directory
            .lineage_events
            .iter()
            .find(|page| {
                page.event_ids
                    .iter()
                    .any(|id| fixture.expected.traversed_event_ids.contains(id))
            })
            .unwrap();
        source.corrupt_member(page.member);
        let expected = Execution::StorageUnavailable(RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Lineage {
                first: LineageId(page.first_node),
                last: LineageId(page.last_node),
            },
            cause: Cause::IntegrityMismatch,
        });
        assert_eq!(first.read(&fixture, execution), Err(expected));
        source.repair();
        let reads = || {
            fixture
                .source
                .member_reads
                .lock()
                .unwrap()
                .iter()
                .filter(|read| **read == page.member)
                .count()
        };
        let failed_reads = reads();
        assert_eq!(
            EventDoor::Historical.read(&fixture, execution),
            Err(expected)
        );
        assert_eq!(
            EventDoor::EmbeddedCheckpoint.read(&fixture, execution),
            Err(expected)
        );
        assert_eq!(
            reads(),
            failed_reads,
            "the repaired source cannot erase proven cell damage"
        );
        let unrelated = LineageId(1);
        let healthy = fixture
            .runtime
            .discover_authored_lineage_history(
                execution,
                &fixture.runtime.main_branch_identity(),
                unrelated,
            )
            .unwrap();
        assert_eq!(healthy.resolved, vec![unrelated]);
        assert!(healthy.traversed_event_ids.is_empty());
    }
}
