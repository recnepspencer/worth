use {
    worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap,
    worth_execution::LeaseRequest, worth_execution::MapKernelFailure,
};

#[test]
fn empty_public_map_uses_no_worker_and_uninstalled_equivalence_is_denied() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let authority = worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: std::num::NonZeroUsize::new(2).unwrap(),
        charged_memory_bytes: Some(32),
    })
    .unwrap();
    let budget = worth_foundational::ExecutionBudget::new(
        std::num::NonZeroUsize::new(2).unwrap(),
        Some(32),
        Some(10),
    );
    let request = |determinism| LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Automatic,
            determinism,
            budget,
        ),
        deadline: None,
        cancellation: worth_execution::CancellationToken::new(),
    };
    assert_eq!(
        authority
            .request_lease(request(
                worth_foundational::DeterminismContract::ContractEquivalent(
                    worth_foundational::EquivalenceContractId::new(7),
                )
            ))
            .unwrap_err(),
        worth_execution::LeaseDenial::EquivalenceContractUnavailable,
    );
    let lease = authority
        .request_lease(request(
            worth_foundational::DeterminismContract::CanonicalBitwise,
        ))
        .unwrap();
    let map =
        ExecutionMap::<u64, u64>::try_from_declared_partitions(Vec::new(), Vec::new()).unwrap();
    let outcome = map.run(Some(&lease), |_: &u64, _| Ok::<_, MapKernelFailure<()>>(1));
    match outcome {
        worth_execution::MapOutcome::Complete { values, report } => {
            assert!(values.is_empty());
            assert_eq!(report.physical().active_workers_high_watermark(), 0);
        }
        worth_execution::MapOutcome::Stopped { reason, .. } => {
            panic!("empty map stopped: {reason:?}")
        }
    }
    let serial_parent = authority
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                budget,
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    let child = serial_parent
        .child(request(
            worth_foundational::DeterminismContract::CanonicalBitwise,
        ))
        .unwrap();
    assert_eq!(
        child.resolved_posture(),
        worth_foundational::ExecutionPosture::Serial
    );
}
