//! Retained-read orchestrators return a region failure before absence or data.
use super::cold_lineage_fixture::{reopened, NodeFixture};
use super::*;
use crate::change_source::RelationalObservationReadDenial;
use crate::execution::RelationalExecutionDenialCause;

#[derive(Clone, Copy, Debug)]
enum Door {
    Entity,
    Relation,
    History,
    VisibleLineage,
}

impl Door {
    fn read(
        self,
        fixture: &NodeFixture,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalObservationReadDenial> {
        let (_, basis) = fixture
            .runtime
            .observe_branch(&fixture.runtime.main_branch_identity())
            .unwrap();
        let observation = basis.observation();
        let entity = fixture.expected.entity_id();
        match self {
            Self::Entity => fixture
                .runtime
                .entity_record_at_observation(execution, &observation, entity)
                .map(|_| ()),
            Self::Relation => fixture
                .runtime
                .relation_record_at_observation(
                    execution,
                    &observation,
                    crate::identity::data::RelationId::new(entity.partition_id, 0, 1),
                )
                .map(|_| ()),
            Self::History => fixture
                .runtime
                .record_history_at_observation(execution, &observation, entity)
                .map(|_| ()),
            Self::VisibleLineage => fixture
                .runtime
                .visible_entities_for_lineages_at_observation(
                    execution,
                    &observation,
                    &[fixture.expected.lineage_id()],
                )
                .map(|_| ()),
        }
    }
}

fn expected(fixture: &NodeFixture) -> RelationalExecutionDenialCause {
    RelationalExecutionDenialCause::StorageUnavailable(
        crate::storage::data::RelationalStorageUnavailable {
            subject: crate::storage::data::RelationalStorageSubject::Region(
                fixture.expected.entity_id().partition_id,
            ),
            cause: Cause::Damaged,
        },
    )
}

#[test]
fn retained_observation_region_guards_preserve_sticky_storage_failure() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    for door in [
        Door::Entity,
        Door::Relation,
        Door::History,
        Door::VisibleLineage,
    ] {
        let fixture = reopened(execution);
        // The visible-lineage door first selects its real node page. The region
        // failure must occur at the exact root read, after that dependency.
        fixture
            .runtime
            .lineage
            .demand_node(execution, fixture.expected.lineage_id())
            .unwrap()
            .unwrap();
        let position = fixture.runtime.history().latest_patch_stream_position();
        let offered = attached_test_medium(&fixture.runtime).offered().len();
        *fixture.source.failed_member.lock().unwrap() = Some(fixture.root_history);
        let denial = RelationalObservationReadDenial::Execution(expected(&fixture));
        assert_eq!(door.read(&fixture, execution), Err(denial), "{door:?}");
        let reads = fixture.source.reads.load(Ordering::Relaxed);
        *fixture.source.failed_member.lock().unwrap() = None;
        assert_eq!(door.read(&fixture, execution), Err(denial), "{door:?}");
        assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
        assert_eq!(
            fixture.runtime.history().latest_patch_stream_position(),
            position
        );
        assert_eq!(
            attached_test_medium(&fixture.runtime).offered().len(),
            offered
        );
    }
}

#[test]
fn lineage_point_guard_preserves_a_cold_mirror_head_failure() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let fixture = reopened(execution);
    *fixture.source.failed_member.lock().unwrap() = Some(fixture.mirror_head);
    let read = || {
        fixture
            .runtime
            .lineage_access()
            .for_record(execution, fixture.expected.entity_id())
    };
    assert_eq!(read(), Err(expected(&fixture)));
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    *fixture.source.failed_member.lock().unwrap() = None;
    assert_eq!(read(), Err(expected(&fixture)));
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
    assert_eq!(fixture.runtime.lineage.node_count(), 0);
}
