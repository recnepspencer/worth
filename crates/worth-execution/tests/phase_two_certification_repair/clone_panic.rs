use super::*;

static CLONES: AtomicUsize = AtomicUsize::new(0);

struct CountClone(u64);

impl Clone for CountClone {
    fn clone(&self) -> Self {
        CLONES.fetch_add(1, Ordering::SeqCst);
        Self(self.0)
    }
}

impl ChargedBytes for CountClone {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl CanonicalBits for CountClone {
    fn canonical_len(&self) -> Option<usize> {
        Some(8)
    }

    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&self.0.to_le_bytes())
    }
}

struct PanicClone(u64);
impl Clone for PanicClone {
    fn clone(&self) -> Self {
        panic!("clone must be contained")
    }
}
impl ChargedBytes for PanicClone {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl CanonicalBits for PanicClone {
    fn canonical_len(&self) -> Option<usize> {
        Some(8)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&self.0.to_le_bytes())
    }
}

#[test]
fn certification_clone_panics_are_typed_failures() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = lease(100, 1 << 20, DeterminismContract::CanonicalBitwise);
    let reduced = map(1).certify_reduce(
        &lease,
        1,
        |value, _| Ok::<_, MapKernelFailure<()>>(PanicClone(*value)),
        PanicClone(0),
        |left, right| PanicClone(left.0 + right.0),
        8,
        0,
    );
    assert!(matches!(
        reduced,
        Err(ReduceCertificationFailure::Mismatch(OracleMismatch::Stop))
    ));

    let mut decompose = ExecutionDecompose::try_new(
        vec![PartitionIdentity::new(1)],
        PanicClone(0),
        |left: &PanicClone, right: &PanicClone| PanicClone(left.0 + right.0),
        8,
        0,
        8,
        128,
        0,
    )
    .unwrap();
    let decomposed = decompose.certify(
        &lease,
        1,
        &map(1),
        DecomposeKernelEditions {
            interior: 1,
            interface: 1,
            back: 1,
        },
        |value, _| {
            Ok::<_, MapKernelFailure<()>>(InteriorResult {
                interior: *value,
                contribution: PanicClone(*value),
            })
        },
        |_, _| {
            Ok::<_, MapKernelFailure<()>>(InterfaceSolution {
                solution: 0_u64,
                slices: vec![0_u64],
            })
        },
        |_, _| Ok::<_, MapKernelFailure<()>>(0_u64),
    );
    assert!(matches!(
        decomposed,
        Err(DecomposeCertificationFailure::Mismatch(
            OracleMismatch::Stop
        ))
    ));
}

#[test]
fn cancelled_certification_does_not_clone_before_admission() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    CLONES.store(0, Ordering::SeqCst);
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let lease = authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), 1 << 20, 100),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap();
    let reduced = map(1).certify_reduce(
        &lease,
        1,
        |value, _| Ok::<_, MapKernelFailure<()>>(CountClone(*value)),
        CountClone(0),
        |left, right| CountClone(left.0 + right.0),
        8,
        0,
    );
    assert!(matches!(
        reduced,
        Err(ReduceCertificationFailure::Mismatch(OracleMismatch::Stop))
    ));
    assert_eq!(CLONES.load(Ordering::SeqCst), 0);

    let mut decomposition = ExecutionDecompose::try_new(
        vec![PartitionIdentity::new(1)],
        CountClone(0),
        |left: &CountClone, right: &CountClone| CountClone(left.0 + right.0),
        8,
        0,
        8,
        128,
        0,
    )
    .unwrap();
    let decomposed = decomposition.certify(
        &lease,
        1,
        &map(1),
        DecomposeKernelEditions {
            interior: 1,
            interface: 1,
            back: 1,
        },
        |value, _| {
            Ok::<_, MapKernelFailure<()>>(InteriorResult {
                interior: *value,
                contribution: CountClone(*value),
            })
        },
        |_, _| {
            Ok::<_, MapKernelFailure<()>>(InterfaceSolution {
                solution: 0_u64,
                slices: vec![0_u64],
            })
        },
        |_, _| Ok::<_, MapKernelFailure<()>>(0_u64),
    );
    assert!(matches!(
        decomposed,
        Err(DecomposeCertificationFailure::Mismatch(
            OracleMismatch::Stop
        ))
    ));
    assert_eq!(CLONES.load(Ordering::SeqCst), 0);
}
