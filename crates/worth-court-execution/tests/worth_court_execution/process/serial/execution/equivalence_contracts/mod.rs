#[allow(unused_imports)]
use crate::equivalence_contracts::isolated_context::*;

use {
    worth_execution::compare_canonical_values, worth_execution::ExecutionAuthorityConfig,
    worth_execution::ExecutionMap, worth_execution::LeaseRequest,
    worth_execution::MapKernelFailure, worth_execution::MapPartition,
};

#[test]
fn predicates_install_once_and_leases_carry_exact_contract_to_children() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let config = ExecutionAuthorityConfig {
        max_workers: std::num::NonZeroUsize::new(2).unwrap(),
        charged_memory_bytes: Some(4096),
    };
    assert_eq!(
        worth_execution::ExecutionAuthority::try_construct_with_equivalences(
            config,
            [predicate(1, 1), predicate(1, 2)],
        )
        .unwrap_err(),
        worth_execution::ConstructionDenial::DuplicateEquivalenceContract(
            worth_foundational::EquivalenceContractId::new(1)
        ),
    );
    assert_eq!(
        worth_execution::ExecutionAuthority::try_construct_with_equivalences(
            config,
            [predicate(1, 1), predicate(2, 1)],
        )
        .unwrap_err(),
        worth_execution::ConstructionDenial::DuplicateEquivalenceIdentity([1; 32]),
    );

    let predicate = worth_execution::EquivalencePredicate::new(
        worth_foundational::EquivalenceContractId::new(1),
        [1; 32],
        |left, right| {
            let Ok(left): Result<[u8; 8], _> = left.try_into() else {
                return false;
            };
            let Ok(right): Result<[u8; 8], _> = right.try_into() else {
                return false;
            };
            (f64::from_le_bytes(left) - f64::from_le_bytes(right)).abs() <= 0.01
        },
    );
    assert_eq!(
        predicate.id(),
        worth_foundational::EquivalenceContractId::new(1)
    );
    assert_eq!(predicate.identity(), [1; 32]);
    let authority =
        worth_execution::ExecutionAuthority::try_construct_with_equivalences(config, [predicate])
            .expect("invalid installation must not consume process authority");

    let installed = worth_foundational::DeterminismContract::ContractEquivalent(
        worth_foundational::EquivalenceContractId::new(1),
    );
    let unknown = worth_foundational::DeterminismContract::ContractEquivalent(
        worth_foundational::EquivalenceContractId::new(2),
    );
    assert_eq!(
        authority.request_lease(request(unknown)).unwrap_err(),
        worth_execution::LeaseDenial::EquivalenceContractUnavailable,
    );
    let parent = authority.request_lease(request(installed)).unwrap();
    let child = parent.child(request(installed)).unwrap();
    assert_eq!(child.policy().determinism(), installed);
    assert_eq!(
        parent.child(request(unknown)).unwrap_err(),
        worth_execution::LeaseDenial::EquivalenceContractUnavailable,
    );
    assert_eq!(
        parent
            .child(request(
                worth_foundational::DeterminismContract::CanonicalBitwise
            ))
            .unwrap_err(),
        worth_execution::LeaseDenial::EquivalenceContractUnavailable,
    );

    let identity = worth_foundational::PartitionIdentity::new(1);
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
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let equivalent = map.certify(&parent, 7, |_, _| {
        let value = if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
            1.0
        } else {
            1.001
        };
        Ok::<_, MapKernelFailure<()>>(value)
    });
    assert!(
        matches!(equivalent, Ok(worth_execution::MapOutcome::Complete { values, .. }) if values == vec![1.001])
    );

    let bitwise = authority
        .request_lease(request(
            worth_foundational::DeterminismContract::CanonicalBitwise,
        ))
        .unwrap();
    let calls = std::sync::atomic::AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&bitwise, 7, |_, _| {
            let value = if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                1.0
            } else {
                1.001
            };
            Ok::<_, MapKernelFailure<()>>(value)
        }),
        Err(worth_execution::OracleMismatch::Values)
    ));

    let comparison = std::sync::Mutex::new(None);
    let outcome = map.run(Some(&parent), |_, _| {
        *comparison.lock().unwrap() = Some(compare_canonical_values(&bitwise, &1_u64, &1_u64));
        Ok::<_, MapKernelFailure<()>>(())
    });
    assert_eq!(
        *comparison.lock().unwrap(),
        Some(Err(worth_execution::OracleMismatch::Stop))
    );
    assert!(matches!(
        outcome,
        worth_execution::MapOutcome::Stopped { .. }
    ));

    let cancellation = worth_execution::CancellationSource::new();
    cancellation.cancel();
    let cancelled = authority
        .request_lease(LeaseRequest {
            cancellation: cancellation.token(),
            ..request(worth_foundational::DeterminismContract::CanonicalBitwise)
        })
        .unwrap();
    assert_eq!(
        compare_canonical_values(&cancelled, &1_u64, &1_u64),
        Err(worth_execution::OracleMismatch::Stop)
    );
}
