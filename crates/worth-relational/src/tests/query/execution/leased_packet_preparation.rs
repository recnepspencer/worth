use std::num::NonZeroUsize;

use worth_execution::{CancellationToken, LeaseRequest, MapKernelFailure, MapStop};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

use super::*;
use crate::facade::runtime::QueryReadExecutionStop;

fn tight_lease() -> worth_execution::ExecutionResourceLease<'static> {
    lease(64 << 10, 100_000)
}

fn lease(memory: u64, work: u64) -> worth_execution::ExecutionResourceLease<'static> {
    crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), memory, work),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}

#[test]
fn any_of_encoding_checks_work_and_memory_before_packet_clones() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    create_entity_in_partition(&runtime, "value", PartitionId(7));
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime.read_truth().query_plan_context(&snapshot).unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "bounded-any-of-preparation".to_owned(),
                context_id: context,
                scope: QueryScope::EntityFieldAnyOf {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                    values: Arc::from([string_aspect_value(&"x".repeat(32_000))]),
                    partition_scope: Some(Arc::from([PartitionId(7)])),
                },
                locality: QueryLocalityClass::PartitionBounded {
                    partitions: Arc::from([PartitionId(7)]),
                },
                ordering: QueryOrderingContract::CanonicalEntityIdOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(9_176_307),
                target_count_hint: 1,
            },
        )
        .unwrap();
    assert!(matches!(
        runtime
            .read_truth()
            .execute_query_plan_with_lease(plan.clone(), &lease(1 << 20, 1)),
        Err(QueryReadExecutionStop::PreparationStopped {
            reason: MapStop::WorkExhausted { .. },
            ..
        })
    ));
    assert!(matches!(
        runtime
            .read_truth()
            .execute_query_plan_with_lease(plan, &tight_lease()),
        Err(QueryReadExecutionStop::PreparationStopped {
            reason: MapStop::Failure {
                cause: MapKernelFailure::ResultCapacityExceeded,
                ..
            },
            ..
        })
    ));
}

#[test]
fn explicit_and_traversal_packet_copies_deny_before_allocation() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let entity = create_entity_in_partition(&runtime, "seed", PartitionId(7));
    let snapshot = runtime.visibility_authority().snapshot();
    let explicit = planned_explicit_query(
        &runtime,
        &snapshot,
        "large-explicit-preparation",
        vec![RecordRef::Entity(entity); 1024],
    );
    let explicit_stop = runtime
        .read_truth()
        .execute_query_plan_with_lease(explicit, &tight_lease());
    assert!(matches!(
        explicit_stop,
        Err(QueryReadExecutionStop::PreparationStopped {
            reason: MapStop::Failure {
                cause: MapKernelFailure::ResultCapacityExceeded,
                ..
            },
            ..
        })
    ));

    let context = runtime.read_truth().query_plan_context(&snapshot).unwrap();
    let traversal = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "large-traversal-preparation".to_owned(),
                context_id: context,
                scope: QueryScope::OutgoingNeighborhood {
                    seeds: Arc::from(vec![entity; 1024]),
                    relation_kind_scope: None,
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(9_176_306),
                target_count_hint: 1024,
            },
        )
        .unwrap();
    let traversal_stop = runtime
        .read_truth()
        .execute_query_plan_with_lease(traversal, &tight_lease());
    assert!(matches!(
        traversal_stop,
        Err(QueryReadExecutionStop::PreparationStopped {
            reason: MapStop::Failure {
                cause: MapKernelFailure::ResultCapacityExceeded,
                ..
            },
            ..
        })
    ));
}

#[test]
fn scoped_scan_does_not_charge_unrelated_partitions() {
    let runtime = runtime_with_test_schema();
    create_entity_in_partition(&runtime, "target", PartitionId(7));
    let scoped_read = || {
        let snapshot = runtime.visibility_authority().snapshot();
        let context = runtime.read_truth().query_plan_context(&snapshot).unwrap();
        let plan = runtime
            .read_truth()
            .plan_query_packet(
                &snapshot,
                PlannedQueryPacket {
                    label: "scoped-read".to_owned(),
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
                    plan_key: DeterministicQueryPlanKey(9_176_308),
                    target_count_hint: 0,
                },
            )
            .unwrap();
        runtime
            .read_truth()
            .execute_query_plan_with_lease_report(plan, &lease(1 << 20, 100_000))
            .unwrap()
            .unwrap()
    };
    let before = scoped_read();
    for partition in 100..120 {
        create_entity_in_partition(&runtime, "other", PartitionId(partition));
    }
    let after = scoped_read();
    assert_eq!(after.report.charged_work(), before.report.charged_work());
    assert_eq!(after.outcome.result, before.outcome.result);
}
