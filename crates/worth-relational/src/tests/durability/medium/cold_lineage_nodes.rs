//! Node reads use genuine node-range members, independently of record heads.
use super::cold_lineage_fixture::{reopened, NodeFixture, POPULATION};
use super::*;
use crate::execution::RelationalExecutionDenialCause;
use crate::history::data::BranchId;
use crate::lineage::data::{HistoricalResolutionBoundednessBasis, RecordHistoryRequest};

#[test]
fn a_cold_lineage_point_read_loads_one_node_page_and_no_region_history() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let fixture = reopened(execution);
    let commits = fixture.source.commit_reads.lock().unwrap().len();
    let observed = fixture
        .runtime
        .lineage_access()
        .for_record(execution, fixture.expected.entity_id())
        .unwrap()
        .unwrap();
    assert_eq!(observed, fixture.expected);
    assert!(fixture.runtime.lineage.node_count() <= 256);
    assert!(
        fixture.runtime.lineage.node_count() < POPULATION / 2,
        "point demand leaves unrelated node pages cold"
    );
    let reads = fixture.source.member_reads.lock().unwrap();
    assert_eq!(
        reads
            .iter()
            .filter(|locator| **locator == fixture.page.member)
            .count(),
        1
    );
    assert!(
        !reads.contains(&fixture.region_history),
        "record association requires Head only"
    );
    assert_eq!(
        fixture.source.commit_reads.lock().unwrap().len(),
        commits,
        "node lookup does not replay envelopes"
    );
}

#[derive(Clone, Copy, Debug)]
enum NodeDoor {
    Point,
    RecordHistory,
    AspectHistory,
    AspectHistoryTrace,
    Graph,
    Divergence,
    ObservationVisible,
}

impl NodeDoor {
    fn read(
        self,
        fixture: &NodeFixture,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalExecutionDenialCause> {
        match self {
            Self::Graph => fixture.runtime.lineage_access().graph(execution,
                crate::lineage::data::LineageGraphRequest {
                    branch_id: BranchId("main".into()),
                    traversal_basis: crate::lineage::data::LineageGraphTraversalBasis::FullBranchGraphMaterialization,
                }).map(|_| ()),
            Self::Divergence => fixture.runtime.lineage_access().divergence_between_branches(execution,
                crate::lineage::data::LineageDivergenceRequest {
                    left_branch: BranchId("main".into()), right_branch: BranchId("main".into()),
                    traversal_basis: crate::lineage::data::LineageDivergenceTraversalBasis::FullBranchGraphComparison,
                }).map(|_| ()),
            Self::ObservationVisible => {
                let identity = fixture.runtime.main_branch_identity();
                let (_, basis) = fixture.runtime.observe_branch(&identity).unwrap();
                fixture.runtime.visible_entities_for_lineages_at_observation(execution,
                    &basis.observation(), &[fixture.expected.lineage_id()])
                    .map(|_| ()).map_err(|denial| match denial {
                        crate::change_source::RelationalObservationReadDenial::Execution(cause) => cause,
                        crate::change_source::RelationalObservationReadDenial::ForeignObservation => panic!("fixture owns its observation"),
                    })
            },
            Self::Point => fixture
                .runtime
                .lineage_access()
                .for_record(execution, fixture.expected.entity_id())
                .map(|_| ()),
            Self::RecordHistory => fixture
                .runtime
                .lineage_access()
                .resolve_record_history(
                    execution,
                    RecordHistoryRequest {
                        branch_id: BranchId("main".into()),
                        entity_id: fixture.expected.entity_id(),
                        boundedness_basis:
                            HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
                    },
                )
                .map(|_| ()),
            Self::AspectHistory => fixture
                .runtime
                .history()
                .lineage_entity_aspect_history(
                    execution,
                    &BranchId("main".into()),
                    fixture.expected.lineage_id(),
                    None,
                )
                .map(|_| ()),
            Self::AspectHistoryTrace => fixture
                .runtime
                .history()
                .lineage_entity_aspect_history_with_trace(
                    execution,
                    &BranchId("main".into()),
                    fixture.expected.lineage_id(),
                    None,
                )
                .map(|_| ()),
        }
    }
}

#[test]
fn every_node_guard_preserves_a_sticky_typed_page_failure_without_publication() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    for door in [
        NodeDoor::Point,
        NodeDoor::RecordHistory,
        NodeDoor::AspectHistory,
        NodeDoor::AspectHistoryTrace,
        NodeDoor::Graph,
        NodeDoor::Divergence,
        NodeDoor::ObservationVisible,
    ] {
        let fixture = reopened(execution);
        fixture
            .runtime
            .acquire_partition_edition()
            .demand_partition_head(execution, fixture.expected.entity_id().partition_id)
            .unwrap();
        // Isolate node admission: these history doors also need their actual
        // canonical bodies, which are admitted through the production cell.
        for artifact in fixture.runtime.history.commit_artifacts() {
            artifact.demand_envelope(execution, 1 << 30).unwrap();
        }
        assert_eq!(fixture.runtime.lineage.node_count(), 0);
        let position = fixture.runtime.history().latest_patch_stream_position();
        let offered = attached_test_medium(&fixture.runtime).offered().len();
        *fixture.source.failed_member.lock().unwrap() = Some(fixture.page.member);
        let expected = RelationalExecutionDenialCause::StorageUnavailable(
            crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Lineage {
                    first: crate::identity::data::LineageId(fixture.page.first),
                    last: crate::identity::data::LineageId(fixture.page.last),
                },
                cause: Cause::Damaged,
            },
        );
        assert_eq!(door.read(&fixture, execution), Err(expected), "{door:?}");
        let reads = fixture.source.reads.load(Ordering::Relaxed);
        let admitted_nodes = fixture.runtime.lineage.node_count();
        assert!(
            fixture
                .runtime
                .lineage
                .node(fixture.expected.lineage_id())
                .is_none(),
            "damaged page admits no node"
        );
        *fixture.source.failed_member.lock().unwrap() = None;
        assert_eq!(door.read(&fixture, execution), Err(expected), "{door:?}");
        assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
        assert_eq!(fixture.runtime.lineage.node_count(), admitted_nodes);
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
fn simultaneous_lineage_point_reads_share_one_page_load() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let fixture = reopened(execution);
    fixture
        .runtime
        .acquire_partition_edition()
        .demand_partition_head(execution, fixture.expected.entity_id().partition_id)
        .unwrap();
    let start = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        let workers = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    let backing = crate::test_execution_policy::serial_request();
                    let execution = &worth_execution::ExecutionRequest::serial(&backing);
                    start.wait();
                    assert_eq!(
                        fixture
                            .runtime
                            .lineage_access()
                            .for_record(execution, fixture.expected.entity_id())
                            .unwrap(),
                        Some(fixture.expected.clone())
                    );
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().unwrap();
        }
    });
    assert_eq!(
        fixture
            .source
            .member_reads
            .lock()
            .unwrap()
            .iter()
            .filter(|locator| **locator == fixture.page.member)
            .count(),
        1,
        "one cold page load serves all concurrent point operations"
    );
}
