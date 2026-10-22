use std::num::NonZeroUsize;
use std::{collections::BTreeMap, sync::Barrier};
use worth_execution::{
    CancellationToken, ConstructionDenial, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionMap, ExecutionRequest, LeaseDenial, LeaseRequest, MapKernelFailure, MapOutcome,
};
use worth_foundational::{
    DeterminismContract, ExecutionPosture, ExecutionRequestPolicy, PartitionIdentity,
};

#[test]
fn worker_authority_cannot_be_reconstructed_after_final_drop() {
    let config = ExecutionAuthorityConfig::for_workers(NonZeroUsize::new(4).unwrap());
    let authority = ExecutionAuthority::try_construct(config).unwrap();
    assert_eq!(
        ExecutionAuthority::try_construct(config).unwrap_err(),
        ConstructionDenial::AlreadyConstructed
    );
    let request = |width| LeaseRequest {
        policy: ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            NonZeroUsize::new(width).unwrap(),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    };
    let lease = authority.request_lease(request(4)).unwrap();
    assert!(matches!(
        lease.child(request(5)),
        Err(LeaseDenial::WorkerLimitExceedsParent)
    ));
    // Both attempts need every admitted worker. A leaked parent or child claim
    // would prevent the second barrier from being reached by four workers.
    for _ in 0..2 {
        let barrier = Barrier::new(4);
        let map = ExecutionMap::<_, ()>::from_native_partitions(
            (1..=4)
                .map(|id| (PartitionIdentity::new(id), id))
                .collect::<BTreeMap<_, _>>(),
        )
        .unwrap();
        let (outcome, physical) = ExecutionRequest::leased(&lease)
            .run_native(|active| {
                map.run_owned_native(active, |value, context| {
                    assert!(!context.has_accounting());
                    context.poll()?;
                    barrier.wait();
                    Ok::<_, MapKernelFailure<()>>(value)
                })
            })
            .unwrap();
        let MapOutcome::Complete { values, report } = outcome else {
            panic!("native worker run refused")
        };
        assert_eq!(values, vec![1, 2, 3, 4]);
        assert_eq!(report.accounted_cost(), None);
        assert_eq!(physical.active_workers_high_watermark(), 4);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
    drop(lease);
    drop(authority);
    assert_eq!(
        ExecutionAuthority::try_construct(config).unwrap_err(),
        ConstructionDenial::AlreadyConstructed
    );
}
