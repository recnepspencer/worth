use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

use worth_execution::{
    CancellationToken, CanonicalBits, ChargedBytes, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionMap, LeaseRequest, MapKernelFailure, MapOutcome, MapPartition, MapStop,
    OracleMismatch,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

fn bits<T: CanonicalBits>(value: &T) -> Vec<u8> {
    let mut output = Vec::new();
    assert!(value.visit_canonical_bits(&mut |chunk| {
        output.extend_from_slice(chunk);
        true
    }));
    assert_eq!(value.canonical_len(), Some(output.len()));
    output
}

#[derive(Clone, Copy)]
struct DeclaredBits {
    length: usize,
    bytes: &'static [u8],
}

impl ChargedBytes for DeclaredBits {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl CanonicalBits for DeclaredBits {
    fn canonical_len(&self) -> Option<usize> {
        Some(self.length)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(self.bytes)
    }
}

#[test]
fn canonical_encoding_preserves_floating_point_bits_and_field_boundaries() {
    assert_ne!(bits(&0.0_f64), bits(&-0.0_f64));

    let nan = f64::from_bits(0x7ff8_0000_0000_0042);
    let other_nan = f64::from_bits(0x7ff8_0000_0000_0043);
    assert_eq!(bits(&nan), bits(&nan));
    assert_ne!(bits(&nan), bits(&other_nan));

    assert_ne!(
        bits(&("a".to_owned(), "bc".to_owned())),
        bits(&("ab".to_owned(), "c".to_owned())),
    );
}

#[test]
fn oracle_certifies_identical_nan_bits_and_rejects_different_value_bits() {
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
                ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), 1024, 10),
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
                ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), 1, 10),
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
