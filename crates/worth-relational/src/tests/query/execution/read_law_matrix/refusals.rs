//! Literal absence and refusal values at both production read doors.
use super::*;
use crate::facade::indexes::{
    BoundedEntityFieldLookupDenialKind as Kind, BoundedEntityFieldLookupRequest,
    BoundedIndexParityMode, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::runtime::QueryReadExecutionStop;
use worth_execution::{
    CancellationSource, ExecutionWorkCeiling, MapKernelFailure, MapKernelStop, MapStop,
    SerialMemoryBudget, SerialRequest,
};
use worth_foundational::{ExecutionPhysicalReport, ExecutionReport, PartitionIdentity};

#[test]
fn runtime_read_law_plan_absence_and_stopped_request() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let world = fixture::World::new(&setup, false);
    let case = cases::table(&world).remove(0);
    let other = fixture::World::new(&setup, false);
    let mut unavailable = world.plan(&case);
    unavailable.snapshot = other.snapshot;
    let mut absent = world.plan(&case);
    absent.packet.context_id.version_id = VersionId(6);
    for plan in [unavailable, absent] {
        let (answer, report) = setup
            .run(ExecutionWorkCeiling::new(100_000), |_| {
                world.runtime.read_truth().execute_query_plan(plan, &setup)
            })
            .unwrap();
        assert!(answer.unwrap().is_none());
        assert_eq!(report.charged_work(), 0);
    }
    let cancellation = CancellationSource::new();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(MEMORY), cancellation.token(), None);
    cancellation.cancel();
    let stopped = world
        .runtime
        .read_truth()
        .execute_query_plan(world.plan(&case), &ExecutionRequest::serial(&host))
        .unwrap_err();
    assert_eq!(
        stopped,
        QueryReadExecutionStop::PreparationStopped {
            reason: MapStop::Failure {
                identity: PartitionIdentity::new(1),
                cause: MapKernelFailure::Stop(MapKernelStop::Cancelled)
            },
            report: ExecutionReport::new(
                ExecutionPosture::Serial,
                0,
                0,
                ExecutionPhysicalReport::new(0, 0, None, 0, 0)
            ),
        }
    );
}

#[test]
fn runtime_read_law_bounded_index_refusal_values() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let world = fixture::World::new(&setup, false);
    let other = fixture::World::new(&setup, false);
    let wrong = world
        .runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(91),
            name: "wrong-kind".into(),
            kind: DerivedIndexKind::RelationField {
                field_locator: world.label.clone(),
            },
            branch_scoped: false,
        });
    for (snapshot, index, expected) in [
        (
            world.snapshot.clone(),
            DerivedIndexId(u64::MAX),
            Kind::IndexNotInstalled,
        ),
        (world.snapshot.clone(), wrong.index_id, Kind::WrongIndexKind),
        (other.snapshot, wrong.index_id, Kind::SnapshotUnavailable),
    ] {
        let request = BoundedEntityFieldLookupRequest::new(
            snapshot,
            index,
            KindId(1),
            world.name.clone(),
            string_aspect_value("a"),
            2,
        )
        .unwrap();
        let (denial, report) = setup
            .run(ExecutionWorkCeiling::new(100_000), |_| {
                world
                    .runtime
                    .index_access()
                    .execute_bounded_entity_field_lookup(
                        request,
                        BoundedIndexParityMode::Production,
                        &setup,
                    )
            })
            .unwrap();
        let denial = denial.unwrap_err();
        assert_eq!(denial.kind(), expected);
        assert_eq!(denial.index_id(), index);
        assert_eq!(denial.examined_entry_count(), 0);
        assert_eq!(report.charged_work(), 0);
    }
}
