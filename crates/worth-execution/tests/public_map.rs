use std::num::NonZeroUsize;

use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionMap, LeaseDenial,
    LeaseRequest, MapKernelFailure, MapOutcome,
};
use worth_foundational::{
    DeterminismContract, EquivalenceContractId, ExecutionBudget, ExecutionPosture,
    ExecutionRequestPolicy,
};

#[test]
fn empty_public_map_uses_no_worker_and_uninstalled_equivalence_is_denied() {
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(2).unwrap(),
        charged_memory_bytes: Some(32),
    })
    .unwrap();
    let budget = ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), 32, 10);
    let request = |determinism| LeaseRequest {
        policy: ExecutionRequestPolicy::new(ExecutionPosture::Automatic, determinism, budget),
        deadline: None,
        cancellation: CancellationToken::new(),
    };
    assert_eq!(
        authority
            .request_lease(request(DeterminismContract::ContractEquivalent(
                EquivalenceContractId::new(7),
            )))
            .unwrap_err(),
        LeaseDenial::EquivalenceContractUnavailable,
    );
    let lease = authority
        .request_lease(request(DeterminismContract::CanonicalBitwise))
        .unwrap();
    let map =
        ExecutionMap::<u64, u64>::try_from_declared_partitions(Vec::new(), Vec::new()).unwrap();
    let outcome = map.run(Some(&lease), |_: &u64, _| Ok::<_, MapKernelFailure<()>>(1));
    match outcome {
        MapOutcome::Complete { values, report } => {
            assert!(values.is_empty());
            assert_eq!(report.physical().active_workers_high_watermark(), 0);
        }
        MapOutcome::Stopped { reason, .. } => panic!("empty map stopped: {reason:?}"),
    }
    let serial_parent = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                budget,
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let child = serial_parent
        .child(request(DeterminismContract::CanonicalBitwise))
        .unwrap();
    assert_eq!(child.resolved_posture(), ExecutionPosture::Serial);
}
