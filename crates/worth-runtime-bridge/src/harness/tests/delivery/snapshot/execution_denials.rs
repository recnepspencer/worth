use super::super::build_runtime;
use crate::facade::{
    BridgeDeliveryErrorKind, BridgeDeliveryReceipt, BridgeExecutionDenial, BridgeRouteRequest,
    BridgeSignalInvalidationDelivery, InvalidationSink, SignalBridgeSinkError,
};
use crate::harness::fixtures::NamedTruth;
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationToken, ExecutionRequest, ExecutionResourceLease, ExecutionScan,
    ExecutionWorkCeiling, LeaseDenial, LeaseRequest, MapKernelFailure, MapStop, ScanOutcome,
};
use worth_foundational::{
    DeterminismContract, EquivalenceContractId, ExecutionBudget, ExecutionPosture,
    ExecutionRequestPolicy, PartitionIdentity,
};

#[derive(Clone, Copy)]
enum Refusal {
    Workers,
    MemoryLimit,
    Work,
    Memory,
    Overflow,
    Nested,
    Equivalence,
}
struct RefusingSink(Refusal);

fn refused_scan(
    lease: Option<&ExecutionResourceLease<'_>>,
    state: u64,
    scratch: u64,
) -> LeaseDenial {
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).unwrap();
    match scan.run(lease, (), state, 0, 0, scratch, |_, _, _| {
        Ok::<_, MapKernelFailure<()>>(((), ()))
    }) {
        ScanOutcome::Stopped {
            reason: MapStop::Admission(denial),
            ..
        } => denial,
        _ => panic!("execution admission must refuse this operation"),
    }
}

impl InvalidationSink for RefusingSink {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        request
            .in_scope(|lease| {
                let lease = lease.expect("the caller's lease is required");
                let child = |workers, memory, work, determinism| {
                    lease
                        .child(LeaseRequest {
                            policy: ExecutionRequestPolicy::new(
                                ExecutionPosture::Automatic,
                                determinism,
                                ExecutionBudget::new(
                                    NonZeroUsize::new(workers).unwrap(),
                                    Some(memory),
                                    Some(work),
                                ),
                            ),
                            cancellation: CancellationToken::new(),
                            deadline: None,
                        })
                        .unwrap_err()
                };
                let denial = match self.0 {
                    Refusal::Workers => child(3, 4096, 10, DeterminismContract::CanonicalBitwise),
                    Refusal::MemoryLimit => {
                        child(2, 4097, 10, DeterminismContract::CanonicalBitwise)
                    }
                    Refusal::Work => child(2, 4096, 11, DeterminismContract::CanonicalBitwise),
                    Refusal::Memory => refused_scan(Some(lease), 0, 4097),
                    Refusal::Overflow => refused_scan(Some(lease), u64::MAX, 0),
                    Refusal::Nested => {
                        ExecutionWorkCeiling::new(Some(10))
                            .run(lease, || refused_scan(None, 0, 0))
                            .unwrap()
                            .0
                    }
                    Refusal::Equivalence => child(
                        2,
                        4096,
                        10,
                        DeterminismContract::ContractEquivalent(EquivalenceContractId::new(91)),
                    ),
                };
                Err(denial.into())
            })
            .map_err(|denial| {
                SignalBridgeSinkError::new(
                    crate::adapter::SignalBridgeSinkErrorKind::Execution(denial.into()),
                    "request admission refused",
                )
            })?
    }
}

#[test]
fn every_sink_resource_cause_survives_delivery_to_the_caller() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    for refusal in [
        Refusal::Workers,
        Refusal::MemoryLimit,
        Refusal::Work,
        Refusal::Memory,
        Refusal::Overflow,
        Refusal::Nested,
        Refusal::Equivalence,
    ] {
        let truth = NamedTruth::new();
        let (commit_a, _observation) = truth.commit_name(execution, "alice");
        let source = truth.source.clone();

        let runtime = build_runtime(source, RefusingSink(refusal), vec![NamedTruth::mapping()]);
        let route = runtime
            .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a.clone()), execution)
            .unwrap();
        let lease = crate::snapshot::test_execution_lease(CancellationToken::new());
        let error = runtime
            .deliver_invalidation(route, worth_execution::ExecutionRequest::leased(&lease))
            .unwrap_err();
        let BridgeDeliveryErrorKind::ExecutionDenied(denial) = error.kind() else {
            panic!("the caller must receive a typed resource denial: {error:?}");
        };
        use BridgeExecutionDenial as Own;
        match refusal {
            Refusal::Workers => assert_eq!(denial, Own::WorkerLimitExceedsParent),
            Refusal::MemoryLimit => assert_eq!(denial, Own::MemoryLimitExceedsParent),
            Refusal::Work => assert_eq!(denial, Own::WorkLimitExceedsParent),
            Refusal::Memory => {
                assert!(matches!(
                    denial,
                    Own::MemoryExhausted(worth_execution::MemoryLimitDenial::Capacity {
                        requested, admitted, ..
                    }) if requested > 4096 && admitted <= 4096
                ))
            }
            Refusal::Overflow => assert_eq!(denial, Own::ChargedBytesOverflow),
            Refusal::Nested => assert_eq!(denial, Own::UnrelatedNestedLease),
            Refusal::Equivalence => assert_eq!(denial, Own::EquivalenceContractUnavailable),
        }
    }
}

fn serial_request(memory: u64) -> worth_execution::SerialRequest {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, Some(memory), Some(1_000)),
    );
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&policy),
        CancellationToken::new(),
        None,
    )
}

fn serial_delivery(
    request: &worth_execution::SerialRequest,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::facade::BridgeDeliveryError {
    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let source = truth.source.clone();

    let sink = crate::harness::fixtures::RecordingSignalBridgeSink::default();
    let runtime = build_runtime(source, sink.clone(), vec![NamedTruth::mapping()]);
    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a.clone()), execution)
        .unwrap();
    let error = runtime
        .deliver_invalidation(route, ExecutionRequest::serial(request))
        .unwrap_err();
    assert!(sink.last_delivery().is_none());
    error
}

#[test]
fn serial_delivery_refuses_memory_before_sink_delivery() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let error = serial_delivery(&serial_request(1), resource_request);
    assert!(matches!(
        error.kind(),
        BridgeDeliveryErrorKind::ExecutionDenied(BridgeExecutionDenial::MemoryExhausted(
            worth_execution::MemoryLimitDenial::Capacity { admitted: 1, .. }
        ))
    ));
}

#[test]
fn serial_delivery_preserves_cancellation_as_its_own_cause() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let source = worth_execution::CancellationSource::new();
    source.cancel();
    let mut request = serial_request(4096);
    request = request.with_cancellation(source.token());
    assert_eq!(
        serial_delivery(&request, resource_request).kind(),
        BridgeDeliveryErrorKind::ExecutionDenied(BridgeExecutionDenial::Cancelled)
    );
}

#[test]
fn serial_delivery_preserves_elapsed_deadline_as_its_own_cause() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let mut request = serial_request(4096);
    let time = worth_execution::MonotonicTimeController::new().unwrap();
    let held = time.hold();
    request = request.with_deadline(Some(time.clock().deadline_at(held)));
    assert_eq!(
        serial_delivery(&request, resource_request).kind(),
        BridgeDeliveryErrorKind::ExecutionDenied(BridgeExecutionDenial::DeadlineElapsed)
    );
}

#[test]
fn historical_ancestor_selection_stops_at_the_serial_work_ceiling() {
    use crate::facade::{BridgeTruthViewEvaluationRequest, TruthBranchIdentity};

    let host = crate::snapshot::test_serial_request();
    let execution = ExecutionRequest::serial(&host);
    let truth = NamedTruth::new();
    let (ancestor, _ancestor) = truth.commit_name(execution, "earlier");
    let (head, _observation) = truth.commit_name(execution, "later");
    let _head = truth.bind_main_head();
    let runtime = build_runtime(
        truth.source.clone(),
        crate::harness::fixtures::RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, None, Some(1)),
    );
    let serial =
        worth_execution::SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    let limited = ExecutionRequest::serial(&serial);
    let branch = TruthBranchIdentity::from_relational_branch_id("main");
    runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(branch.clone(), head),
            limited,
        )
        .expect("selecting the exact head needs no ancestry walk");

    let error = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(branch, ancestor),
            limited,
        )
        .err()
        .expect("the same ceiling must stop the ancestry walk");
    assert_eq!(
        error.kind(),
        BridgeDeliveryErrorKind::ExecutionDenied(BridgeExecutionDenial::WorkCeiling)
    );
}
