use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

use worth_execution::{
    compare_canonical_values, CancellationSource, CancellationToken, ConstructionDenial,
    EquivalencePredicate, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionMap, LeaseDenial,
    LeaseRequest, MapKernelFailure, MapOutcome, MapPartition, OracleMismatch,
};
use worth_foundational::{
    DeterminismContract, EquivalenceContractId, ExecutionBudget, ExecutionPosture,
    ExecutionRequestPolicy, PartitionIdentity,
};

fn request(determinism: DeterminismContract) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            determinism,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), 4096, 10),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

fn predicate(id: u64, identity: u8) -> EquivalencePredicate {
    EquivalencePredicate::new(
        EquivalenceContractId::new(id),
        [identity; 32],
        |left, right| left == right,
    )
}

#[test]
fn predicates_install_once_and_leases_carry_exact_contract_to_children() {
    let config = ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(2).unwrap(),
        charged_memory_bytes: Some(4096),
    };
    assert_eq!(
        ExecutionAuthority::try_construct_with_equivalences(
            config,
            [predicate(1, 1), predicate(1, 2)],
        )
        .unwrap_err(),
        ConstructionDenial::DuplicateEquivalenceContract(EquivalenceContractId::new(1)),
    );
    assert_eq!(
        ExecutionAuthority::try_construct_with_equivalences(
            config,
            [predicate(1, 1), predicate(2, 1)],
        )
        .unwrap_err(),
        ConstructionDenial::DuplicateEquivalenceIdentity([1; 32]),
    );

    let predicate =
        EquivalencePredicate::new(EquivalenceContractId::new(1), [1; 32], |left, right| {
            let Ok(left): Result<[u8; 8], _> = left.try_into() else {
                return false;
            };
            let Ok(right): Result<[u8; 8], _> = right.try_into() else {
                return false;
            };
            (f64::from_le_bytes(left) - f64::from_le_bytes(right)).abs() <= 0.01
        });
    assert_eq!(predicate.id(), EquivalenceContractId::new(1));
    assert_eq!(predicate.identity(), [1; 32]);
    let authority = ExecutionAuthority::try_construct_with_equivalences(config, [predicate])
        .expect("invalid installation must not consume process authority");

    let installed = DeterminismContract::ContractEquivalent(EquivalenceContractId::new(1));
    let unknown = DeterminismContract::ContractEquivalent(EquivalenceContractId::new(2));
    assert_eq!(
        authority.request_lease(request(unknown)).unwrap_err(),
        LeaseDenial::EquivalenceContractUnavailable,
    );
    let parent = authority.request_lease(request(installed)).unwrap();
    let child = parent.child(request(installed)).unwrap();
    assert_eq!(child.policy().determinism(), installed);
    assert_eq!(
        parent.child(request(unknown)).unwrap_err(),
        LeaseDenial::EquivalenceContractUnavailable,
    );
    assert_eq!(
        parent
            .child(request(DeterminismContract::CanonicalBitwise))
            .unwrap_err(),
        LeaseDenial::EquivalenceContractUnavailable,
    );

    let identity = PartitionIdentity::new(1);
    let map = ExecutionMap::<(), u64>::try_from_declared_partitions(
        vec![identity],
        vec![MapPartition {
            identity,
            value: (),
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        }],
    )
    .unwrap();
    let calls = AtomicUsize::new(0);
    let equivalent = map.certify(&parent, 7, |_, _| {
        let value = if calls.fetch_add(1, Ordering::SeqCst) == 0 {
            1.0
        } else {
            1.001
        };
        Ok::<_, MapKernelFailure<()>>(value)
    });
    assert!(matches!(equivalent, Ok(MapOutcome::Complete { values, .. }) if values == vec![1.001]));

    let bitwise = authority
        .request_lease(request(DeterminismContract::CanonicalBitwise))
        .unwrap();
    let calls = AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&bitwise, 7, |_, _| {
            let value = if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                1.0
            } else {
                1.001
            };
            Ok::<_, MapKernelFailure<()>>(value)
        }),
        Err(OracleMismatch::Values)
    ));

    let comparison = std::sync::Mutex::new(None);
    let outcome = map.run(Some(&parent), |_, _| {
        *comparison.lock().unwrap() = Some(compare_canonical_values(&bitwise, &1_u64, &1_u64));
        Ok::<_, MapKernelFailure<()>>(())
    });
    assert_eq!(*comparison.lock().unwrap(), Some(Err(OracleMismatch::Stop)));
    assert!(matches!(outcome, MapOutcome::Stopped { .. }));

    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let cancelled = authority
        .request_lease(LeaseRequest {
            cancellation: cancellation.token(),
            ..request(DeterminismContract::CanonicalBitwise)
        })
        .unwrap();
    assert_eq!(
        compare_canonical_values(&cancelled, &1_u64, &1_u64),
        Err(OracleMismatch::Stop)
    );
}
