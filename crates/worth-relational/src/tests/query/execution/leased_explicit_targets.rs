use std::num::NonZeroUsize;

use worth_execution::{
    CancellationSource, CancellationToken, LeaseRequest, MapKernelFailure, MapStop,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

use super::*;
use crate::facade::runtime::QueryReadExecutionStop;

const MEMORY: u64 = 1 << 26;

mod packet_identity;

fn lease(
    width: usize,
    memory: u64,
    cancellation: CancellationToken,
) -> worth_execution::ExecutionResourceLease<'static> {
    lease_with_work(width, memory, 100_000, cancellation)
}

fn lease_with_work(
    width: usize,
    memory: u64,
    work: u64,
    cancellation: CancellationToken,
) -> worth_execution::ExecutionResourceLease<'static> {
    crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(width).unwrap(), memory, work),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}

#[test]
fn leased_explicit_read_preserves_packet_identity_and_reports_stops() {
    let runtime = runtime_with_test_schema();
    let targets = vec![
        RecordRef::Entity(create_entity_in_partition(&runtime, "a", PartitionId(7))),
        RecordRef::Entity(create_entity_in_partition(&runtime, "b", PartitionId(11))),
        RecordRef::Entity(create_entity_in_partition(&runtime, "c", PartitionId(13))),
    ];
    let snapshot = runtime.visibility_authority().snapshot();
    let plan = planned_explicit_query(&runtime, &snapshot, "leased-explicit", targets);
    let expected = runtime
        .read_truth()
        .execute_query_plan(plan.clone())
        .unwrap();
    assert_eq!(expected.complexity.packet_count, 3);

    let width_one = lease(1, MEMORY, CancellationToken::new());
    let serial = runtime
        .read_truth()
        .execute_query_plan_with_lease_report(plan.clone(), &width_one)
        .unwrap()
        .unwrap();
    let width_four = lease(4, MEMORY, CancellationToken::new());
    let parallel = runtime
        .read_truth()
        .execute_query_plan_with_lease_report(plan.clone(), &width_four)
        .unwrap()
        .unwrap();
    assert_eq!(serial.outcome.result, expected.result);
    assert_eq!(parallel.outcome.result, expected.result);
    assert_eq!(serial.outcome.complexity, expected.complexity);
    assert_eq!(parallel.outcome.complexity, expected.complexity);
    assert!(serial.report.charged_work() > 3);
    assert_eq!(parallel.report.charged_work(), serial.report.charged_work());

    let almost_complete = lease_with_work(
        1,
        MEMORY,
        serial.report.charged_work() - 1,
        CancellationToken::new(),
    );
    assert!(matches!(
        runtime
            .read_truth()
            .execute_query_plan_with_lease(plan.clone(), &almost_complete),
        Err(QueryReadExecutionStop::CompletionStopped {
            reason: MapStop::WorkExhausted { .. },
            ..
        })
    ));

    let cancelled = CancellationSource::new();
    cancelled.cancel();
    let cancelled_lease = lease(4, MEMORY, cancelled.token());
    let stopped = runtime
        .read_truth()
        .execute_query_plan_with_lease(plan.clone(), &cancelled_lease)
        .unwrap_err();
    assert!(matches!(stopped,
        QueryReadExecutionStop::PreparationStopped {
            reason: MapStop::Failure {
                identity,
                cause: MapKernelFailure::Stop(worth_execution::MapKernelStop::Cancelled),
            },
            ..
        } if identity == PartitionIdentity::new(1)
    ));

    let constrained = lease(4, 1024, CancellationToken::new());
    assert!(matches!(
        runtime
            .read_truth()
            .execute_query_plan_with_lease(plan, &constrained),
        Err(QueryReadExecutionStop::PreparationStopped {
            reason: MapStop::Admission(worth_execution::LeaseDenial::MemoryExhausted(_))
                | MapStop::Failure {
                    cause: MapKernelFailure::ResultCapacityExceeded,
                    ..
                },
            ..
        })
    ));
}

#[test]
fn leased_field_filter_charges_captured_comparison_bytes() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    create_entity_in_partition(&runtime, "small", PartitionId(7));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime.read_truth().query_plan_context(&snapshot).unwrap();
    let packet = PlannedQueryPacket {
        label: "charged-field-filter".to_owned(),
        context_id: context,
        scope: QueryScope::EntityFieldEquals {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            value: string_aspect_value(&"x".repeat(32_000)),
            partition_scope: Some(Arc::from([PartitionId(7)])),
        },
        locality: QueryLocalityClass::PartitionBounded {
            partitions: Arc::from([PartitionId(7)]),
        },
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(9_176_301),
        target_count_hint: 0,
    };
    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .unwrap();
    let expected = runtime
        .read_truth()
        .execute_query_plan(plan.clone())
        .unwrap();
    let wide = lease(4, MEMORY, CancellationToken::new());
    let actual = runtime
        .read_truth()
        .execute_query_plan_with_lease(plan.clone(), &wide)
        .unwrap()
        .unwrap();
    assert_eq!(actual.result, expected.result);

    let tight = lease(4, 16 << 10, CancellationToken::new());
    assert!(matches!(
        runtime
            .read_truth()
            .execute_query_plan_with_lease(plan, &tight),
        Err(QueryReadExecutionStop::PreparationStopped {
            reason: MapStop::Admission(worth_execution::LeaseDenial::MemoryExhausted(_))
                | MapStop::Failure {
                    cause: MapKernelFailure::ResultCapacityExceeded,
                    ..
                },
            ..
        })
    ));
}

#[test]
fn leased_kind_scan_stops_at_a_record_checkpoint() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    for ordinal in 0..24 {
        create_entity_in_partition(&runtime, &format!("entity-{ordinal}"), PartitionId(7));
    }
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime.read_truth().query_plan_context(&snapshot).unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "leased-kind-scan-ceiling".to_owned(),
                context_id: context,
                scope: QueryScope::EntityKindScan {
                    kind_id: KindId(1),
                    partition_scope: Some(Arc::from([PartitionId(7)])),
                },
                locality: QueryLocalityClass::PartitionBounded {
                    partitions: Arc::from([PartitionId(7)]),
                },
                ordering: QueryOrderingContract::CanonicalEntityIdOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(9_176_302),
                target_count_hint: 0,
            },
        )
        .unwrap();
    let unleased = runtime
        .read_truth()
        .execute_query_plan(plan.clone())
        .unwrap();
    assert_eq!(unleased.result.entities.len(), 24);

    let constrained = lease_with_work(1, 1 << 20, 10, CancellationToken::new());
    let actual = runtime
        .read_truth()
        .execute_query_plan_with_lease(plan, &constrained);
    assert!(
        matches!(
            &actual,
            Err(QueryReadExecutionStop::PreparationStopped {
                reason: MapStop::WorkExhausted { .. },
                ..
            }) | Err(QueryReadExecutionStop::PacketStopped {
                reason: MapStop::WorkExhausted { .. },
                ..
            })
        ),
        "{actual:?}"
    );
}

#[test]
fn leased_traversal_stops_inside_high_fanout() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let seed = create_entity_in_partition(&runtime, "seed", PartitionId(7));
    for ordinal in 0..24 {
        let neighbor =
            create_entity_in_partition(&runtime, &format!("neighbor-{ordinal}"), PartitionId(11));
        create_relation_in_partition(
            &runtime,
            seed,
            neighbor,
            &format!("edge-{ordinal}"),
            PartitionId(13),
        );
    }
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime.read_truth().query_plan_context(&snapshot).unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "leased-traversal-ceiling".to_owned(),
                context_id: context,
                scope: QueryScope::OutgoingNeighborhood {
                    seeds: Arc::from([seed]),
                    relation_kind_scope: Some(Arc::from([KindId(2)])),
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(9_176_303),
                target_count_hint: 1,
            },
        )
        .unwrap();
    let unleased = runtime
        .read_truth()
        .execute_query_plan(plan.clone())
        .unwrap();
    assert_eq!(unleased.result.relations.len(), 24);

    let constrained = lease_with_work(1, 1 << 20, 10, CancellationToken::new());
    let actual = runtime
        .read_truth()
        .execute_query_plan_with_lease(plan, &constrained);
    assert!(
        matches!(
            &actual,
            Err(QueryReadExecutionStop::PreparationStopped {
                reason: MapStop::WorkExhausted { .. },
                ..
            }) | Err(QueryReadExecutionStop::PacketStopped {
                reason: MapStop::WorkExhausted { .. },
                ..
            })
        ),
        "{actual:?}"
    );
}

#[test]
fn leased_kind_scan_rejects_one_record_larger_than_its_packet_ceiling() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    create_entity_in_partition(&runtime, &"x".repeat(32_000), PartitionId(7));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime.read_truth().query_plan_context(&snapshot).unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "leased-oversized-record".to_owned(),
                context_id: context,
                scope: QueryScope::EntityKindScan {
                    kind_id: KindId(1),
                    partition_scope: Some(Arc::from([PartitionId(7)])),
                },
                locality: QueryLocalityClass::PartitionBounded {
                    partitions: Arc::from([PartitionId(7)]),
                },
                ordering: QueryOrderingContract::CanonicalEntityIdOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(9_176_304),
                target_count_hint: 0,
            },
        )
        .unwrap();
    let constrained = lease(1, 64 << 10, CancellationToken::new());
    assert!(matches!(
        runtime
            .read_truth()
            .execute_query_plan_with_lease(plan, &constrained),
        Err(QueryReadExecutionStop::PacketStopped {
            reason: MapStop::Failure {
                cause: MapKernelFailure::ResultCapacityExceeded,
                ..
            },
            ..
        })
    ));
}

#[test]
fn leased_any_of_scan_matches_serial_with_reversed_values() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    create_entity_in_partition(&runtime, "alpha", PartitionId(7));
    create_entity_in_partition(&runtime, "omega", PartitionId(7));
    create_entity_in_partition(&runtime, "other", PartitionId(7));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime.read_truth().query_plan_context(&snapshot).unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "leased-any-of-order".to_owned(),
                context_id: context,
                scope: QueryScope::EntityFieldAnyOf {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                    values: Arc::from([string_aspect_value("omega"), string_aspect_value("alpha")]),
                    partition_scope: Some(Arc::from([PartitionId(7)])),
                },
                locality: QueryLocalityClass::PartitionBounded {
                    partitions: Arc::from([PartitionId(7)]),
                },
                ordering: QueryOrderingContract::CanonicalEntityIdOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(9_176_305),
                target_count_hint: 2,
            },
        )
        .unwrap();
    let expected = runtime
        .read_truth()
        .execute_query_plan(plan.clone())
        .unwrap();
    let actual = runtime
        .read_truth()
        .execute_query_plan_with_lease(plan.clone(), &lease(1, 1 << 20, CancellationToken::new()))
        .unwrap()
        .unwrap();
    assert_eq!(actual.result, expected.result);
    assert_eq!(actual.result.entities.len(), 2);
}
