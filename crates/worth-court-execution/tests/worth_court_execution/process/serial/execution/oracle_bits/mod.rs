#[allow(unused_imports)]
pub(crate) use {std::sync::atomic::AtomicUsize, worth_execution::CancellationToken, crate::oracle_bits::ordinary_context::DeclaredBits, worth_foundational::DeterminismContract, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_foundational::ExecutionBudget, worth_execution::ExecutionMap, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition, worth_execution::MapStop, std::num::NonZeroUsize, worth_execution::OracleMismatch, std::sync::atomic::Ordering, worth_foundational::PartitionIdentity, crate::oracle_bits::ordinary_context::bits};

#[test]
fn oracle_certifies_identical_nan_bits_and_rejects_different_value_bits() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let authority = worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: std::num::NonZeroUsize::new(2).unwrap(),
        charged_memory_bytes: Some(1024),
    })
    .unwrap();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::new(2).unwrap(), Some(1024), Some(10)),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    let identity = worth_foundational::PartitionIdentity::new(1);
    let map = ExecutionMap::<(), u64>::try_from_declared_partitions(
        vec![identity],
        vec![MapPartition {
            identity,
            value: (),
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 8,
        }],
    )
    .unwrap();

    for (length, bytes) in [(0, &[1_u8][..]), (2, &[1_u8][..]), (2048, &[1_u8][..])] {
        assert!(matches!(
            map.certify(&lease, 7, |_, _| Ok::<_, MapKernelFailure<u8>>(
                DeclaredBits { length, bytes }
            )),
            Err(worth_execution::OracleMismatch::Stop)
        ));
    }
    let calls = std::sync::atomic::AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&lease, 7, |_, _| {
            let length = if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                1
            } else {
                2
            };
            Ok::<_, MapKernelFailure<u8>>(DeclaredBits {
                length,
                bytes: &[1],
            })
        }),
        Err(worth_execution::OracleMismatch::Values)
    ));
    let calls = std::sync::atomic::AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&lease, 7, |_, _| {
            let bytes = if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                &[1_u8][..]
            } else {
                &[][..]
            };
            Ok::<_, MapKernelFailure<u8>>(DeclaredBits { length: 1, bytes })
        }),
        Err(worth_execution::OracleMismatch::Values)
    ));

    let nan_bits = 0x7ff8_0000_0000_0042;
    let calls = std::sync::atomic::AtomicUsize::new(0);
    assert!(matches!(map
        .certify(&lease, 7, |_, _| {
            calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok::<f64, MapKernelFailure<f64>>(f64::from_bits(nan_bits))
        }), Ok(worth_execution::MapOutcome::Complete { values, .. }) if values[0].to_bits() == nan_bits));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);

    let denied_lease = authority
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::new(2).unwrap(), Some(1), Some(10)),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    let calls = std::sync::atomic::AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&denied_lease, 7, |_, _| {
            calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok::<f64, MapKernelFailure<f64>>(1.0)
        }),
        Err(worth_execution::OracleMismatch::Stop)
    ));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);

    let calls = std::sync::atomic::AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&lease, 7, |_, _| {
            let bits = if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                nan_bits
            } else {
                nan_bits + 1
            };
            Ok::<f64, MapKernelFailure<f64>>(f64::from_bits(bits))
        }),
        Err(worth_execution::OracleMismatch::Values)
    ));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);

    let calls = std::sync::atomic::AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&lease, 7, |_, _| {
            let value = if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                0.0_f64
            } else {
                -0.0_f64
            };
            Ok::<f64, MapKernelFailure<f64>>(value)
        }),
        Err(worth_execution::OracleMismatch::Values)
    ));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);

    let calls = std::sync::atomic::AtomicUsize::new(0);
    assert!(matches!(map
        .certify(&lease, 7, |_, _| {
            calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err::<f64, _>(worth_execution::MapKernelFailure::Domain(f64::from_bits(nan_bits)))
        }), Ok(worth_execution::MapOutcome::Stopped { reason: worth_execution::MapStop::Failure { cause: worth_execution::MapKernelFailure::Domain(value), .. }, .. }) if value.to_bits() == nan_bits));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);

    let calls = std::sync::atomic::AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&lease, 7, |_, _| {
            let bits = if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                nan_bits
            } else {
                nan_bits + 1
            };
            Err::<f64, _>(worth_execution::MapKernelFailure::Domain(f64::from_bits(bits)))
        }),
        Err(worth_execution::OracleMismatch::Stop)
    ));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
}
