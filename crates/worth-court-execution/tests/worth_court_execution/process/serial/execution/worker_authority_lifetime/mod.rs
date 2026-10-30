use std::num::NonZeroUsize;
use {std::collections::BTreeMap, std::sync::Barrier};
use {
    worth_execution::CancellationToken, worth_execution::ConstructionDenial,
    worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig,
    worth_execution::ExecutionMap, worth_execution::ExecutionRequest, worth_execution::LeaseDenial,
    worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome,
};
use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionPosture,
    worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity,
};

#[test]
fn worker_authority_cannot_be_reconstructed_after_final_drop() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let config = worth_execution::ExecutionAuthorityConfig::for_workers(
        std::num::NonZeroUsize::new(4).unwrap(),
    );
    let authority = worth_execution::ExecutionAuthority::try_construct(config).unwrap();
    assert_eq!(
        worth_execution::ExecutionAuthority::try_construct(config).unwrap_err(),
        worth_execution::ConstructionDenial::AlreadyConstructed
    );
    let request = |width| LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::for_workers(
            worth_foundational::ExecutionPosture::Automatic,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(width).unwrap(),
        ),
        deadline: None,
        cancellation: worth_execution::CancellationToken::new(),
    };
    let lease = authority.request_lease(request(4)).unwrap();
    assert!(matches!(
        lease.child(request(5)),
        Err(worth_execution::LeaseDenial::WorkerLimitExceedsParent)
    ));
    // Both attempts need every admitted worker. A leaked parent or child claim
    // would prevent the second barrier from being reached by four workers.
    for _ in 0..2 {
        let barrier = std::sync::Barrier::new(4);
        let map = ExecutionMap::<_, ()>::from_native_partitions(
            (1..=4)
                .map(|id| (worth_foundational::PartitionIdentity::new(id), id))
                .collect::<BTreeMap<_, _>>(),
        )
        .unwrap();
        let (outcome, physical) = worth_execution::ExecutionRequest::leased(&lease)
            .run_native(|active| {
                map.run_owned_native(active, |value, context| {
                    assert!(!context.has_accounting());
                    context.poll()?;
                    barrier.wait();
                    Ok::<_, MapKernelFailure<()>>(value)
                })
            })
            .unwrap();
        let worth_execution::MapOutcome::Complete { values, report } = outcome else {
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
        worth_execution::ExecutionAuthority::try_construct(config).unwrap_err(),
        worth_execution::ConstructionDenial::AlreadyConstructed
    );
}
