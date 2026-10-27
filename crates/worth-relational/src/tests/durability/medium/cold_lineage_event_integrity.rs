//! Corrupt actual producer bytes, independently of the source's error classifier.
use super::cold_lineage_event_fixture::{reopened_with_source, request};
use super::cold_lineage_event_matrix::EventDoor;
use super::*;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberCustody};
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};
use std::sync::Mutex;

#[derive(Debug)]
struct ChangedBytes {
    producer: Arc<MemberSource>,
    target: Mutex<Option<RelationalDurableMemberLocator>>,
    corrupt: AtomicBool,
}
impl RelationalDurableColdSource for ChangedBytes {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        self.producer.retain_members(members)
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let mut bytes = self.producer.read_member(locator, limit)?;
        if self.corrupt.load(Ordering::Acquire) && *self.target.lock().unwrap() == Some(locator) {
            *bytes.last_mut().unwrap() ^= 1;
        }
        Ok(bytes)
    }
    fn read_commit(
        &self,
        position: crate::publication::patch::data::PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.producer.read_commit(position, limit)
    }
}
#[test]
fn changed_event_bytes_damage_only_their_page_and_share_the_refusal_with_export() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for first in [EventDoor::Historical, EventDoor::EmbeddedCheckpoint] {
        let mut changed = None;
        let fixture = reopened_with_source(execution, |producer| {
            let source = Arc::new(ChangedBytes {
                producer,
                target: Mutex::new(None),
                corrupt: AtomicBool::new(false),
            });
            changed = Some(source.clone());
            source
        });
        let source = changed.unwrap();
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
        *source.target.lock().unwrap() = Some(page.member);
        source.corrupt.store(true, Ordering::Release);
        let expected = Execution::StorageUnavailable(RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Lineage {
                first: crate::identity::data::LineageId(page.first_node),
                last: crate::identity::data::LineageId(page.last_node),
            },
            cause: Cause::IntegrityMismatch,
        });
        assert_eq!(first.read(&fixture, execution), Err(expected));
        source.corrupt.store(false, Ordering::Release);
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
        let unrelated = crate::identity::data::LineageId(1);
        let healthy = fixture
            .runtime
            .lineage_access()
            .resolve_historical_lineage(execution, request(unrelated))
            .unwrap();
        assert_eq!(healthy.resolved, vec![unrelated]);
        assert!(healthy.traversed_event_ids.is_empty());
    }
}
