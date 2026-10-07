use super::super::{
    build_runtime, commit_a, committed_patch, patch_a, registration, snapshot, snapshot_a,
};
use crate::facade::{
    BridgeDeliveryErrorKind, BridgeDeliveryReceipt, BridgeExecutionDenial, BridgeRouteRequest,
    BridgeSignalInvalidationDelivery, InvalidationSink, SignalBridgeSinkError,
};
use crate::harness::fixtures::InMemoryRelationalBridgeSource;
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
                                    memory,
                                    work,
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
                        ExecutionWorkCeiling::new(10)
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
            .map_err(|denial| SignalBridgeSinkError::Execution(denial.into()))?
    }
}

#[test]
fn every_sink_resource_cause_survives_delivery_to_the_caller() {
    for refusal in [
        Refusal::Workers,
        Refusal::MemoryLimit,
        Refusal::Work,
        Refusal::Memory,
        Refusal::Overflow,
        Refusal::Nested,
        Refusal::Equivalence,
    ] {
        let source = InMemoryRelationalBridgeSource::default();
        source.insert_committed_patch(committed_patch(
            commit_a(),
            patch_a(),
            snapshot_a(),
            worth_foundational::facade::FieldKey::new("name".to_owned()).unwrap(),
        ));
        source.insert_snapshot(snapshot(snapshot_a(), "alice"));
        let runtime = build_runtime(source, RefusingSink(refusal), vec![registration()]);
        let route = runtime
            .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a()))
            .unwrap();
        let lease = crate::snapshot::test_execution_lease(CancellationToken::new());
        let error = runtime
            .deliver_invalidation_with_lease(route, &lease)
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
                assert!(matches!(denial, Own::MemoryExhausted(memory) if memory.admitted == 4096))
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
        ExecutionBudget::new(NonZeroUsize::MIN, memory, 1_000),
    );
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&policy),
        CancellationToken::new(),
        None,
    )
}

fn serial_delivery(request: &worth_execution::SerialRequest) -> crate::facade::BridgeDeliveryError {
    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        commit_a(),
        patch_a(),
        snapshot_a(),
        worth_foundational::facade::FieldKey::new("name".to_owned()).unwrap(),
    ));
    source.insert_snapshot(snapshot(snapshot_a(), "alice"));
    let sink = crate::harness::fixtures::RecordingSignalBridgeSink::default();
    let runtime = build_runtime(source, sink.clone(), vec![registration()]);
    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a()))
        .unwrap();
    let error = runtime
        .deliver_invalidation_with_request(route, ExecutionRequest::serial(request))
        .unwrap_err();
    assert!(sink.last_delivery().is_none());
    error
}

#[test]
fn serial_delivery_refuses_memory_before_sink_delivery() {
    let error = serial_delivery(&serial_request(1));
    assert!(
        matches!(error.kind(), BridgeDeliveryErrorKind::ExecutionDenied(
        BridgeExecutionDenial::MemoryExhausted(memory)) if memory.admitted == 1)
    );
}

#[test]
fn serial_delivery_preserves_cancellation_as_its_own_cause() {
    let source = worth_execution::CancellationSource::new();
    source.cancel();
    let mut request = serial_request(4096);
    request = request.with_cancellation(source.token());
    assert_eq!(
        serial_delivery(&request).kind(),
        BridgeDeliveryErrorKind::ExecutionDenied(BridgeExecutionDenial::Cancelled)
    );
}

#[test]
fn serial_delivery_preserves_elapsed_deadline_as_its_own_cause() {
    let mut request = serial_request(4096);
    request = request.with_deadline(Some(
        std::time::Instant::now() - std::time::Duration::from_secs(1),
    ));
    assert_eq!(
        serial_delivery(&request).kind(),
        BridgeDeliveryErrorKind::ExecutionDenied(BridgeExecutionDenial::DeadlineElapsed)
    );
}
