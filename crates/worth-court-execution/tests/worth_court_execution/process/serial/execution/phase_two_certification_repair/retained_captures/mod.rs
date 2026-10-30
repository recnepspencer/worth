#[allow(unused_imports)]
use crate::phase_two_certification_repair::isolated_context::retained_captures::*;
use {
    crate::phase_two_certification_repair::isolated_context::lease,
    crate::phase_two_certification_repair::isolated_context::map,
    crate::phase_two_certification_repair::isolated_context::TEST_LOCK,
    std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering,
    worth_execution::DecomposeCertificationFailure, worth_execution::DecomposeKernelEditions,
    worth_execution::ExecutionDecompose, worth_execution::ExecutionMap,
    worth_execution::InterfaceSolution, worth_execution::InteriorResult,
    worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition,
    worth_execution::OracleMismatch, worth_foundational::DeterminismContract,
    worth_foundational::PartitionIdentity,
};

#[test]
fn warm_decomposition_admits_both_captured_reducer_copies_before_clone() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    CAPTURE_CLONES.store(0, std::sync::atomic::Ordering::SeqCst);
    let capture = TrackedCapture(vec![0; 100_000]);
    let mut decomposition = worth_execution::ExecutionDecompose::try_new(
        vec![worth_foundational::PartitionIdentity::new(1)],
        0_u64,
        move |left: &u64, right: &u64| {
            std::hint::black_box(capture.touch());
            left + right
        },
        8,
        8,
        8,
        512,
        100_000,
    )
    .unwrap();
    let editions = DecomposeKernelEditions {
        interior: 1,
        interface: 1,
        back: 1,
    };
    decomposition
        .run(None, &map(1), editions, interior, interface, back)
        .expect("serial warm-up");
    let before = CAPTURE_CLONES.load(std::sync::atomic::Ordering::SeqCst);
    assert!(before > 0, "warm-up must clone the captured reducer");
    let tight = lease(
        100,
        150_000,
        worth_foundational::DeterminismContract::CanonicalBitwise,
    );
    let result = decomposition.certify(&tight, 1, &map(1), editions, interior, interface, back);
    assert!(matches!(
        result,
        Err(worth_execution::DecomposeCertificationFailure::Mismatch(
            worth_execution::OracleMismatch::Stop
        ))
    ));
    assert_eq!(
        CAPTURE_CLONES.load(std::sync::atomic::Ordering::SeqCst),
        before
    );
}

#[test]
fn reduction_peak_includes_reference_capture_during_selected_nested_work() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = lease(
        1_000,
        1 << 20,
        worth_foundational::DeterminismContract::CanonicalBitwise,
    );
    let inner: ExecutionMap<u64, u64> =
        worth_execution::ExecutionMap::try_from_declared_partitions(
            vec![worth_foundational::PartitionIdentity::new(9)],
            vec![MapPartition {
                identity: worth_foundational::PartitionIdentity::new(9),
                value: 9_u64,
                read_keys: Vec::new(),
                write_keys: Vec::new(),
                kernel_scratch_bytes: 60_000,
                max_result_bytes: 8,
            }],
        )
        .unwrap();
    let capture = TrackedCapture(vec![0; 100_000]);
    let (_, report, _) = map(1)
        .certify_reduce(
            &lease,
            2,
            |value, _| match inner.run(Some(&lease), |_, context| {
                context.checkpoint(1)?;
                Ok::<_, MapKernelFailure<()>>(1_u64)
            }) {
                worth_execution::MapOutcome::Complete { .. } => Ok(*value),
                _ => Err(MapKernelFailure::<()>::Stop(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
            },
            0_u64,
            move |left: &u64, right: &u64| {
                std::hint::black_box(capture.touch());
                left + right
            },
            8,
            100_000,
        )
        .unwrap_or_else(|_| panic!("wide lease must certify reduction"));
    assert!(report.physical().peak_charged_memory_bytes() >= 260_000);
}
