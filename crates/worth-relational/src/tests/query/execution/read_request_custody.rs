//! The production plan entry spends the caller's borrowed request.
use super::*;
use crate::execution::RelationalExecutionDenialCause;
use crate::query::data::SnapshotPinnedQueryPlan;
use crate::transactions::data::CommitExecutionDenialKind;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

const MEMORY: u64 = 1 << 26;

fn fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (RelationalRuntime, SnapshotPinnedQueryPlan) {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    create_entity_in_partition(execution, &runtime, "left-a", PartitionId(7));
    create_entity_in_partition(execution, &runtime, "left-b", PartitionId(7));
    create_entity_in_partition(execution, &runtime, "right-a", PartitionId(11));
    let snapshot = runtime.visibility_authority().snapshot();
    let context_id = runtime.read_truth().query_plan_context(&snapshot).unwrap();
    let packet = PlannedQueryPacket {
        label: "read-request-custody".to_owned(),
        context_id,
        scope: QueryScope::EntityKindScan {
            kind_id: KindId(1),
            partition_scope: Some(Arc::from([PartitionId(7), PartitionId(11)])),
        },
        locality: QueryLocalityClass::PartitionBounded {
            partitions: Arc::from([PartitionId(7), PartitionId(11)]),
        },
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(9_176_301),
        target_count_hint: 3,
    };
    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .unwrap();
    (runtime, plan)
}

fn lease(
    width: usize,
    work: u64,
    cancellation: CancellationToken,
) -> worth_execution::ExecutionResourceLease<'static> {
    test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(width).unwrap(), MEMORY, Some(work)),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}

#[test]
fn production_kind_scan_spends_its_callers_work_ceiling() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, plan) = fixture(execution);
    let lease = lease(1, 1, CancellationToken::new());
    // Three fixture rows at the owner's one-unit examined-row fee cannot fit one unit.
    let stopped = runtime
        .read_truth()
        .execute_query_plan(plan, &ExecutionRequest::leased(&lease))
        .expect_err("a production kind scan must spend its caller's work");
    assert_eq!(
        stopped.execution_denial().unwrap().kind,
        CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::WorkExhausted)
    );
}

fn serial(cancellation: CancellationToken) -> worth_execution::SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(MEMORY),
        cancellation,
        None,
    )
}

#[test]
fn production_plan_has_equal_bits_and_charged_work_at_every_backing() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, plan) = fixture(execution);
    let host = serial(CancellationToken::new());
    let reference = runtime
        .read_truth()
        .execute_query_plan(plan.clone(), &ExecutionRequest::serial(&host))
        .unwrap()
        .unwrap();
    let work = reference.report.unwrap().charged_work();
    for width in [1, 2, 4] {
        let lease = lease(width, u64::MAX, CancellationToken::new());
        let read = runtime
            .read_truth()
            .execute_query_plan(plan.clone(), &ExecutionRequest::leased(&lease))
            .unwrap()
            .unwrap();
        assert_eq!(
            read.outcome.result, reference.outcome.result,
            "width {width}"
        );
        assert_eq!(
            read.outcome.complexity, reference.outcome.complexity,
            "width {width}"
        );
        assert_eq!(read.report.unwrap().charged_work(), work, "width {width}");
    }
}

#[test]
fn production_plan_cancellation_has_one_typed_cause_at_every_backing() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, plan) = fixture(execution);
    for width in [0, 1, 2, 4] {
        let source = worth_execution::CancellationSource::new();
        let host = serial(source.token());
        let lease = (width != 0).then(|| lease(width, u64::MAX, source.token()));
        let request = match lease.as_ref() {
            Some(lease) => ExecutionRequest::leased(lease),
            None => ExecutionRequest::serial(&host),
        };
        source.cancel();
        let stopped = runtime
            .read_truth()
            .execute_query_plan(plan.clone(), &request)
            .unwrap_err();
        assert!(
            matches!(stopped, crate::facade::runtime::QueryReadExecutionStop::PreparationStopped {
            reason: worth_execution::MapStop::Failure {
                identity, cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::Cancelled),
            }, report,
        } if identity == worth_foundational::PartitionIdentity::new(1) && report.charged_work() == 0),
            "width {width}: {stopped:?}"
        );
    }
}

#[test]
fn serial_scope_admission_precedes_unavailable_snapshot() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let (runtime, plan) = fixture(&setup);
    let other = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let mut unavailable = plan.clone();
    unavailable.snapshot = other.visibility_authority().snapshot();
    let host = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(0),
        CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&host);
    let admission = runtime
        .read_truth()
        .execute_query_plan(plan, &request)
        .unwrap_err();
    assert!(matches!(
        admission,
        crate::runtime::QueryReadExecutionStop::Request(
            worth_execution::WorkCeilingDenial::Admission(
                worth_execution::LeaseDenial::MemoryExhausted(worth_execution::MemoryLimitDenial {
                    admitted: 0,
                    level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
                    ..
                })
            )
        )
    ));
    assert_eq!(
        runtime
            .read_truth()
            .execute_query_plan(unavailable, &request)
            .err(),
        Some(admission)
    );
}
