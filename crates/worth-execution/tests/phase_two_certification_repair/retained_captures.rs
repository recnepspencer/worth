use super::*;

static CAPTURE_CLONES: AtomicUsize = AtomicUsize::new(0);

struct TrackedCapture(Vec<u8>);

impl TrackedCapture {
    fn touch(&self) -> usize {
        self.0.len()
    }
}

impl Clone for TrackedCapture {
    fn clone(&self) -> Self {
        CAPTURE_CLONES.fetch_add(1, Ordering::SeqCst);
        Self(self.0.clone())
    }
}

fn interior(
    value: &u64,
    _: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InteriorResult<u64, u64>, MapKernelFailure<()>> {
    Ok(InteriorResult {
        interior: *value,
        contribution: *value,
    })
}

fn interface(
    value: &u64,
    _: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InterfaceSolution<u64, u64>, MapKernelFailure<()>> {
    Ok(InterfaceSolution {
        solution: *value,
        slices: vec![*value],
    })
}

fn back(
    input: &worth_execution::BackInput<u64, u64>,
    _: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<()>> {
    Ok(input.interior + input.interface_slice)
}

#[test]
fn warm_decomposition_admits_both_captured_reducer_copies_before_clone() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    CAPTURE_CLONES.store(0, Ordering::SeqCst);
    let capture = TrackedCapture(vec![0; 100_000]);
    let mut decomposition = ExecutionDecompose::try_new(
        vec![PartitionIdentity::new(1)],
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
    let before = CAPTURE_CLONES.load(Ordering::SeqCst);
    assert!(before > 0, "warm-up must clone the captured reducer");
    let tight = lease(100, 150_000, DeterminismContract::CanonicalBitwise);
    let result = decomposition.certify(&tight, 1, &map(1), editions, interior, interface, back);
    assert!(matches!(
        result,
        Err(DecomposeCertificationFailure::Mismatch(
            OracleMismatch::Stop
        ))
    ));
    assert_eq!(CAPTURE_CLONES.load(Ordering::SeqCst), before);
}

#[test]
fn reduction_peak_includes_reference_capture_during_selected_nested_work() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = lease(1_000, 1 << 20, DeterminismContract::CanonicalBitwise);
    let inner: ExecutionMap<u64, u64> = ExecutionMap::try_from_declared_partitions(
        vec![PartitionIdentity::new(9)],
        vec![MapPartition {
            identity: PartitionIdentity::new(9),
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
                MapOutcome::Complete { .. } => Ok(*value),
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
