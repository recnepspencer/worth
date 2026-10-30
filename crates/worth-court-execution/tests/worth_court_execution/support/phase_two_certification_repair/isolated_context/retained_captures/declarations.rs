pub(crate) use {
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

pub(crate) static CAPTURE_CLONES: AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub(crate) struct TrackedCapture(pub(crate) Vec<u8>);

impl TrackedCapture {
    pub(crate) fn touch(&self) -> usize {
        self.0.len()
    }
}

impl Clone for TrackedCapture {
    fn clone(&self) -> Self {
        CAPTURE_CLONES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self(self.0.clone())
    }
}

pub(crate) fn interior(
    value: &u64,
    _: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InteriorResult<u64, u64>, MapKernelFailure<()>> {
    Ok(InteriorResult {
        interior: *value,
        contribution: *value,
    })
}

pub(crate) fn interface(
    value: &u64,
    _: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InterfaceSolution<u64, u64>, MapKernelFailure<()>> {
    Ok(InterfaceSolution {
        solution: *value,
        slices: vec![*value],
    })
}

pub(crate) fn back(
    input: &worth_execution::BackInput<u64, u64>,
    _: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<()>> {
    Ok(input.interior + input.interface_slice)
}
