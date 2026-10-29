#[allow(unused_imports)]
pub(crate) use crate::fast::oracle_bits::*;

#[test]
fn oracle_certifies_identical_nan_bits_and_rejects_different_value_bits() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(2).unwrap(),
        charged_memory_bytes: Some(1024),
    })
    .unwrap();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), Some(1024), Some(10)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let identity = PartitionIdentity::new(1);
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
            Err(OracleMismatch::Stop)
        ));
    }
    let calls = AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&lease, 7, |_, _| {
            let length = if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                1
            } else {
                2
            };
            Ok::<_, MapKernelFailure<u8>>(DeclaredBits {
                length,
                bytes: &[1],
            })
        }),
        Err(OracleMismatch::Values)
    ));
    let calls = AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&lease, 7, |_, _| {
            let bytes = if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                &[1_u8][..]
            } else {
                &[][..]
            };
            Ok::<_, MapKernelFailure<u8>>(DeclaredBits { length: 1, bytes })
        }),
        Err(OracleMismatch::Values)
    ));

    let nan_bits = 0x7ff8_0000_0000_0042;
    let calls = AtomicUsize::new(0);
    assert!(matches!(map
        .certify(&lease, 7, |_, _| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok::<f64, MapKernelFailure<f64>>(f64::from_bits(nan_bits))
        }), Ok(MapOutcome::Complete { values, .. }) if values[0].to_bits() == nan_bits));
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    let denied_lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), Some(1), Some(10)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let calls = AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&denied_lease, 7, |_, _| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok::<f64, MapKernelFailure<f64>>(1.0)
        }),
        Err(OracleMismatch::Stop)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    let calls = AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&lease, 7, |_, _| {
            let bits = if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                nan_bits
            } else {
                nan_bits + 1
            };
            Ok::<f64, MapKernelFailure<f64>>(f64::from_bits(bits))
        }),
        Err(OracleMismatch::Values)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    let calls = AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&lease, 7, |_, _| {
            let value = if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                0.0_f64
            } else {
                -0.0_f64
            };
            Ok::<f64, MapKernelFailure<f64>>(value)
        }),
        Err(OracleMismatch::Values)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    let calls = AtomicUsize::new(0);
    assert!(matches!(map
        .certify(&lease, 7, |_, _| {
            calls.fetch_add(1, Ordering::SeqCst);
            Err::<f64, _>(MapKernelFailure::Domain(f64::from_bits(nan_bits)))
        }), Ok(MapOutcome::Stopped { reason: MapStop::Failure { cause: MapKernelFailure::Domain(value), .. }, .. }) if value.to_bits() == nan_bits));
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    let calls = AtomicUsize::new(0);
    assert!(matches!(
        map.certify(&lease, 7, |_, _| {
            let bits = if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                nan_bits
            } else {
                nan_bits + 1
            };
            Err::<f64, _>(MapKernelFailure::Domain(f64::from_bits(bits)))
        }),
        Err(OracleMismatch::Stop)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
