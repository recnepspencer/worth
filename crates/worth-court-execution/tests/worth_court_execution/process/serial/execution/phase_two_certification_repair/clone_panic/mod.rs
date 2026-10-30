#[allow(unused_imports)]
use crate::phase_two_certification_repair::isolated_context::clone_panic::*;
use {
    crate::phase_two_certification_repair::isolated_context::authority,
    crate::phase_two_certification_repair::isolated_context::lease,
    crate::phase_two_certification_repair::isolated_context::map,
    crate::phase_two_certification_repair::isolated_context::TEST_LOCK,
    worth_execution::DecomposeKernelEditions, worth_execution::InterfaceSolution,
    worth_execution::InteriorResult, worth_execution::LeaseRequest,
    worth_execution::MapKernelFailure,
};

#[test]
fn certification_clone_panics_are_typed_failures() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = lease(
        100,
        1 << 20,
        worth_foundational::DeterminismContract::CanonicalBitwise,
    );
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
        Err(worth_execution::ReduceCertificationFailure::Mismatch(
            worth_execution::OracleMismatch::Stop
        ))
    ));

    let mut decompose = worth_execution::ExecutionDecompose::try_new(
        vec![worth_foundational::PartitionIdentity::new(1)],
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
        Err(worth_execution::DecomposeCertificationFailure::Mismatch(
            worth_execution::OracleMismatch::Stop
        ))
    ));
}

#[test]
fn cancelled_certification_does_not_clone_before_admission() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    CLONES.store(0, std::sync::atomic::Ordering::SeqCst);
    let cancellation = worth_execution::CancellationSource::new();
    cancellation.cancel();
    let lease = authority()
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(1).unwrap(),
                    None,
                    Some(100),
                ),
            ),
            deadline: None,
            cancellation: cancellation.token(),
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
        Err(worth_execution::ReduceCertificationFailure::Mismatch(
            worth_execution::OracleMismatch::Stop
        ))
    ));
    assert_eq!(CLONES.load(std::sync::atomic::Ordering::SeqCst), 0);

    let mut decomposition = worth_execution::ExecutionDecompose::try_new(
        vec![worth_foundational::PartitionIdentity::new(1)],
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
        Err(worth_execution::DecomposeCertificationFailure::Mismatch(
            worth_execution::OracleMismatch::Stop
        ))
    ));
    assert_eq!(CLONES.load(std::sync::atomic::Ordering::SeqCst), 0);
}
