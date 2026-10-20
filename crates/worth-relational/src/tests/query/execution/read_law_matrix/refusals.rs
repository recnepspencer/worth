//! Literal absence and refusal values at both production read doors.
use super::*;
use crate::facade::indexes::{
    BoundedEntityFieldLookupDenialKind as Kind, BoundedEntityFieldLookupRequest,
    BoundedIndexParityMode, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::runtime::QueryReadExecutionStop;
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionWorkCeiling, LeaseRequest, MapKernelFailure,
    MapKernelStop, MapStop, SerialMemoryBudget, SerialRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPhysicalReport, ExecutionReport,
    ExecutionRequestPolicy, PartitionIdentity,
};

fn admitted_lease(token: CancellationToken) -> worth_execution::ExecutionResourceLease<'static> {
    test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), None, Some(100_000)),
            ),
            deadline: None,
            cancellation: token,
        })
        .unwrap()
}

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
    for leased in [false, true] {
        let lease = admitted_lease(CancellationToken::new());
        let request = if leased {
            ExecutionRequest::leased(&lease)
        } else {
            setup
        };
        for plan in [unavailable.clone(), absent.clone()] {
            let (answer, report) = request
                .run(ExecutionWorkCeiling::new(Some(100_000)), |_| {
                    world
                        .runtime
                        .read_truth()
                        .execute_query_plan(plan, &request)
                })
                .unwrap();
            assert!(answer.unwrap().is_none());
            assert_eq!(report.charged_work(), 0);
        }
    }
    for leased in [false, true] {
        let cancellation = CancellationSource::new();
        let host =
            SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
        let lease = admitted_lease(cancellation.token());
        let serial = ExecutionRequest::serial(&host);
        let request = if leased {
            ExecutionRequest::leased(&lease)
        } else {
            serial
        };
        cancellation.cancel();
        let stopped = world
            .runtime
            .read_truth()
            .execute_query_plan(world.plan(&case), &request)
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
    for leased in [false, true] {
        let lease = admitted_lease(CancellationToken::new());
        let execution = if leased {
            ExecutionRequest::leased(&lease)
        } else {
            setup
        };
        for (snapshot, index, expected) in [
            (
                world.snapshot.clone(),
                DerivedIndexId(u64::MAX),
                Kind::IndexNotInstalled,
            ),
            (world.snapshot.clone(), wrong.index_id, Kind::WrongIndexKind),
            (
                other.snapshot.clone(),
                wrong.index_id,
                Kind::SnapshotUnavailable,
            ),
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
            let (denial, report) = execution
                .run(ExecutionWorkCeiling::new(Some(100_000)), |_| {
                    world
                        .runtime
                        .index_access()
                        .execute_bounded_entity_field_lookup(
                            request,
                            BoundedIndexParityMode::Production,
                            &execution,
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
}
