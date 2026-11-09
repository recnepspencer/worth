//! Retained-read orchestrators return a region failure before absence or data.
use super::lineage_node_source::{reopened, NodeFixture};
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    change_source::RelationalObservationReadDenial,
    history::BranchId,
    identity::{RelationId, VersionId},
    inspection::{HistoricalInspectionMode, InspectionAvailability},
    storage::{
        RelationalStorageSubject, RelationalStorageUnavailable,
        RelationalStorageUnavailableCause as Cause,
    },
    transactions::RecordRef,
    transactions::RelationalExecutionDenialCause,
};

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
                    RelationId::new(entity.partition_id, 0, 1),
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
    RelationalExecutionDenialCause::StorageUnavailable(RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Region(fixture.expected.entity_id().partition_id),
        cause: Cause::Damaged,
    })
}

#[test]
fn retained_observation_region_guards_preserve_sticky_storage_failure() {
    let backing = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
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
            .immutable_lineage_node(execution, fixture.expected.lineage_id())
            .unwrap()
            .unwrap();
        let position = fixture.runtime.history().latest_patch_stream_position();
        let offered = fixture.medium.offered().len();
        *fixture.source.failed_member.lock().unwrap() = Some({
            let (_, basis) = fixture
                .runtime
                .observe_branch(&fixture.runtime.main_branch_identity())
                .unwrap();
            let selected = fixture
                .runtime
                .history()
                .branch_head_for_observation(&basis.observation())
                .unwrap()
                .unwrap();
            fixture
                .directory
                .roots
                .iter()
                .find(|root| root.commit_id == selected.commit_id)
                .unwrap()
                .regions
                .iter()
                .find(|region| region.partition_id == fixture.expected.entity_id().partition_id)
                .unwrap()
                .history
        });
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
        assert_eq!(fixture.medium.offered().len(), offered);
    }
}

#[test]
fn lineage_point_guard_preserves_a_cold_mirror_head_failure() {
    let backing = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let fixture = reopened(execution);
    *fixture.source.failed_member.lock().unwrap() = Some(
        fixture
            .directory
            .mirror
            .iter()
            .find(|r| r.partition_id == fixture.expected.entity_id().partition_id)
            .unwrap()
            .head,
    );
    let missing_version = VersionId(u64::MAX);
    let reads_before = fixture.source.reads.load(Ordering::Relaxed);
    let open = fixture
        .runtime
        .inspect_what_happened()
        .open_historical_view(
            execution,
            missing_version,
            HistoricalInspectionMode::RetainedOnly,
        )
        .unwrap();
    assert!(open.view.is_none());
    assert_eq!(
        open.availability,
        InspectionAvailability::UnavailableByRetention
    );
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads_before);
    let read = || {
        fixture
            .runtime
            .inspect_what_happened()
            .inspect_historical_record(
                execution,
                &BranchId("main".into()),
                missing_version,
                RecordRef::Entity(fixture.expected.entity_id()),
                HistoricalInspectionMode::RetainedOnly,
            )
    };
    assert_eq!(read(), Err(expected(&fixture)));
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    *fixture.source.failed_member.lock().unwrap() = None;
    assert_eq!(read(), Err(expected(&fixture)));
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
}
